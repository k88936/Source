//! The asset window: the photos a page shows without paging.
//!
//! A window is an interval of the timeline, two anchors either of which can be
//! open, so a subscriber gets the rows inside it once and then only the rows
//! that crossed an anchor or changed inside it. An interval is decided row by
//! row, so what changed is answered from the changes alone: no window is kept
//! here, and a row that leaves the interval leaves the subscriber's copy with
//! it.

use std::future::Future;

use serde::{Deserialize, Serialize};
use source_subscription::{Change, Delta, Row, Value, View};
use ts_rs::TS;

use crate::store::{
    ASSET_FOLDER_ID, ASSET_ID, ASSET_MEDIA_TYPE, ASSET_TAKEN_AT, ASSETS, AssetStore, AssetSummary,
    FolderId, MEDIA_TYPES, TimelineCursor,
};

impl Row for AssetSummary {
    fn id(&self) -> String {
        self.id.0.clone()
    }
}

/// One interval of the timeline: a scope and its two anchors.
///
/// `from` is inclusive and `to` exclusive, so a page cuts the next interval from
/// the key at the bottom of this one and the two neither overlap nor leave a
/// hole. An absent anchor is that end of the timeline.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssetsQuery {
    /// The album to look in; absent means the whole library.
    pub album_id: Option<FolderId>,
    /// The oldest row of the interval, and that end of the timeline when absent.
    pub from: Option<TimelineCursor>,
    /// The newest row the interval stops before, so the whole head when absent.
    pub to: Option<TimelineCursor>,
}

/// A page of the timeline, one interval at a time.
pub struct Assets<C> {
    catalog: C,
}

/// Which side of a change a check reads.
#[derive(Clone, Copy)]
enum Side {
    Before,
    After,
}

impl Side {
    fn value<'a>(self, change: &'a Change, column: &str) -> Option<&'a Value> {
        match self {
            Self::Before => change.old(column),
            Self::After => change.new(column),
        }
    }
}

impl<C: AssetStore> Assets<C> {
    pub fn new(catalog: C) -> Self {
        Self { catalog }
    }

    async fn window(&self, query: &AssetsQuery) -> Result<Vec<AssetSummary>, String> {
        self.catalog
            .asset_window(
                query.album_id.as_ref().map(FolderId::as_str),
                query.from.clone(),
                query.to.clone(),
            )
            .await
            .map_err(|e| e.view_message())
    }

    /// What one interval changed by.
    ///
    /// Decided per row: a side inside the interval is a row the subscriber must
    /// have, and a side that was inside and no longer is a row it must drop. The
    /// rows that stayed are read back, because a row's own columns can change
    /// without it moving.
    ///
    /// A commit that touches one row more than once is answered by that row's
    /// last side, so an id is never on both lists.
    async fn changed(
        &self,
        query: &AssetsQuery,
        changes: &[Change],
    ) -> Result<Delta<AssetSummary>, String> {
        let mut entered: Vec<String> = Vec::new();
        let mut left: Vec<String> = Vec::new();
        for change in changes.iter().filter(|change| change.table == ASSETS.name) {
            let before = query.held(change, Side::Before);
            let after = query.held(change, Side::After);
            if let Some(id) = after {
                left.retain(|held| held.as_str() != id);
                entered.push(id.to_owned());
            } else if let Some(id) = before {
                entered.retain(|held| held.as_str() != id);
                left.push(id.to_owned());
            }
        }

        let mut delta = Delta::new();
        if !entered.is_empty() {
            let ids: Vec<&str> = entered.iter().map(String::as_str).collect();
            delta.inserts = self
                .catalog
                .asset_summaries(&ids)
                .await
                .map_err(|e| e.view_message())?;
        }
        delta.deletes = left;
        Ok(delta)
    }
}

impl AssetsQuery {
    /// The row's id when this side of a change is inside the interval.
    ///
    /// Every condition of a window is a property of one row - scope, kind and
    /// the two anchors - which is what lets a single change say whether a row
    /// entered or left without reading the window back.
    fn held<'a>(&self, change: &'a Change, side: Side) -> Option<&'a str> {
        let media = side.value(change, ASSET_MEDIA_TYPE)?.as_str()?;
        if !MEDIA_TYPES.contains(&media) {
            return None;
        }
        if let Some(album) = &self.album_id {
            let folder = side.value(change, ASSET_FOLDER_ID)?.as_str()?;
            if folder != album.as_str() {
                return None;
            }
        }
        let taken_at = side.value(change, ASSET_TAKEN_AT)?.as_i64()?;
        let id = side.value(change, ASSET_ID)?.as_str()?;
        self.contains(taken_at, id).then_some(id)
    }

    /// Whether a key is inside the interval: `from` inclusive, `to` exclusive.
    fn contains(&self, taken_at: i64, id: &str) -> bool {
        let key = (taken_at, id);
        let after_from = self
            .from
            .as_ref()
            .is_none_or(|from| key >= (from.taken_at, from.id.as_str()));
        let before_to = self
            .to
            .as_ref()
            .is_none_or(|to| key < (to.taken_at, to.id.as_str()));
        after_from && before_to
    }
}

impl<C> View for Assets<C>
where
    C: AssetStore + 'static,
{
    type Query = AssetsQuery;
    type Row = AssetSummary;

    fn snapshot(
        &self,
        query: &AssetsQuery,
    ) -> impl Future<Output = Result<Vec<AssetSummary>, String>> + Send {
        self.window(query)
    }

    fn delta(
        &self,
        query: &AssetsQuery,
        changes: &[Change],
    ) -> impl Future<Output = Result<Delta<AssetSummary>, String>> + Send {
        self.changed(query, changes)
    }
}

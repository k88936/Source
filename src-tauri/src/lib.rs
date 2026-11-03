//! Thin Tauri adapter: argument/error mapping, events, and the `thumb://` /
//! `media://` protocols. Business rules live in `source-core`.

mod bootstrap;
mod commands;
mod credentials;
#[cfg(debug_assertions)]
mod dev;
mod prelude;
mod protocols;
mod state;

pub use credentials::KeyringCredentialStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    dev::isolate_home();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_keyring::init());

    // The simulated CDP endpoint only exists when `--debug-cdp-port` is passed.
    #[cfg(debug_assertions)]
    let builder = match dev::cdp_port() {
        Some(port) => builder.plugin(tauri_plugin_playwright::init_with_config(
            dev::playwright_config(port),
        )),
        None => builder,
    };

    builder
        .register_asynchronous_uri_scheme_protocol("thumb", |ctx, request, responder| {
            protocols::thumb(ctx.app_handle().clone(), request, responder);
        })
        .register_asynchronous_uri_scheme_protocol("media", |ctx, request, responder| {
            protocols::media(ctx.app_handle().clone(), request, responder);
        })
        .register_asynchronous_uri_scheme_protocol("local", |ctx, request, responder| {
            protocols::local_thumb(ctx.app_handle().clone(), request, responder);
        })
        .setup(|app| {
            bootstrap::setup(app)?;
            #[cfg(debug_assertions)]
            dev::open_dev_frontend(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connection::app_info,
            commands::connection::set_server,
            commands::connection::login,
            commands::connection::login_2fa,
            commands::connection::logout,
            commands::connection::session_state,
            commands::connection::refresh_capacity,
            commands::connection::list_remote_folders,
            commands::connection::set_library_root,
            commands::index::start_index,
            commands::index::cancel_index,
            commands::index::index_status,
            commands::catalog::timeline_page,
            commands::catalog::album_list,
            commands::catalog::album_page,
            commands::catalog::asset_detail,
            commands::catalog::search_assets,
            commands::catalog::set_favorite,
            commands::catalog::set_album_cover,
            commands::catalog::set_album_description,
            commands::catalog::move_assets_to_album,
            commands::local::media_permission_state,
            commands::local::request_media_permission,
            commands::local::local_album_list,
            commands::local::local_asset_page,
            commands::uploads::enqueue_uploads,
            commands::uploads::upload_tasks,
            commands::uploads::cancel_upload,
            commands::uploads::retry_upload,
            commands::export::cache_stats,
            commands::export::clear_cache,
            commands::export::export_assets
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

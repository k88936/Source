/**
 * The one place an error becomes a sentence.
 *
 * Rust only sends identity and metadata; every word the user reads lives here.
 * A cause chain is rendered as one line, joined by ": ", because every place
 * that shows it - a toast, a queue row, a page banner - is a single line.
 */
import type {AccountError} from '#lib/api/bindings/AccountError';
import type {AlbumError} from '#lib/api/bindings/AlbumError';
import type {BrowseError} from '#lib/api/bindings/BrowseError';
import type {CapacityError} from '#lib/api/bindings/CapacityError';
import type {CatalogError} from '#lib/api/bindings/CatalogError';
import type {CreateFolderError} from '#lib/api/bindings/CreateFolderError';
import type {ExportError} from '#lib/api/bindings/ExportError';
import type {FolderError} from '#lib/api/bindings/FolderError';
import type {ListError} from '#lib/api/bindings/ListError';
import type {MediaError} from '#lib/api/bindings/MediaError';
import type {MediaFetchError} from '#lib/api/bindings/MediaFetchError';
import type {MetadataError} from '#lib/api/bindings/MetadataError';
import type {MoveError} from '#lib/api/bindings/MoveError';
import type {ProbeError} from '#lib/api/bindings/ProbeError';
import type {RemoteError} from '#lib/api/bindings/RemoteError';
import type {SessionError} from '#lib/api/bindings/SessionError';
import type {SignInError} from '#lib/api/bindings/SignInError';
import type {SyncError} from '#lib/api/bindings/SyncError';
import type {UploadError} from '#lib/api/bindings/UploadError';
import type {UploadUriError} from '#lib/api/bindings/UploadUriError';
import {bundle} from '#lib/i18n/bundle';
import {i18n} from '#lib/i18n/i18n.svelte';

/** Every error a command can report, in the shape the IPC body carries. */
export type AppError =
    | AccountError
    | AlbumError
    | BrowseError
    | FolderError
    | SyncError
    | UploadError
    | ExportError
    | CatalogError
    | MediaError
    | ProbeError
    | SignInError
    | SessionError
    | CapacityError
    | ListError
    | MediaFetchError
    | MetadataError
    | CreateFolderError
    | MoveError
    | UploadUriError
    | RemoteError;

const words = bundle(
    {
        notSignedIn: 'Not signed in',
        sessionExpired: 'The session has expired, sign in again',
        unreachable: 'Cannot reach the server',
        rejected: 'The server refused the request',
        protocol: 'The server answered in a form this client cannot read',
        notCloudreve: 'This address is not a Cloudreve server',
        unauthorized: 'This account is not authorized',
        credentialsRejected: 'Wrong email or password',
        codeRejected: 'Wrong verification code',
        serverAddressInvalid: 'The server address is not valid',
        serverMissing: 'Choose a server first',
        signInMissing: 'Sign in again before continuing',
        folderGone: 'This folder no longer exists on the server',
        assetGone: 'This file no longer exists on the server',
        permission: 'No permission for this file',
        permissionDenied: 'The device library needs permission',
        nameRejected: 'The server refused this name',
        parentGone: 'The parent folder no longer exists',
        alreadyExists: 'A folder with this name already exists',
        quotaExceeded: 'Not enough storage left',
        sourceGone: 'The original file no longer exists on the server',
        destinationGone: 'The target folder no longer exists',
        conflict: 'The target already has a file with this name',
        sourceUnreadable: 'The local file could not be read',
        chunkFailed: 'The upload was interrupted',
        interrupted: 'The upload was stopped',
        sessionExpiredUpload: 'The upload session has expired',
        unavailable: 'The index could not be opened',
        queryFailed: 'The index could not answer',
        decodeFailed: 'A stored value could not be read',
        encodeFailed: 'A value could not be stored',
        platform: 'The platform service failed',
        decodePreviewFailed: 'A preview could not be generated',
        serverAddressEmpty: 'The server address is empty',
        catalog: 'The local index failed',
        albumMissing: 'This album no longer exists',
        assetMissing: 'This asset no longer exists',
        localAlbumMissing: 'This local album no longer exists',
        notAFolder: 'Only a folder can be added as an album',
        listing: 'The folder could not be listed',
        metadata: 'The album details could not be saved',
        move: 'The files could not be moved',
        moveMismatch: 'The server moved a different number of files',
        localMedia: 'The device library failed',
        thumb: 'The thumbnail could not be loaded',
        media: 'The file could not be loaded',
        transport: 'The request failed',
        directoryUnreadable: 'This device folder could not be read',
        queueFailed: 'The file could not be added to the upload queue',
        taskMissing: 'This upload task no longer exists',
        taskNotFailed: 'This upload task is not in a failed state',
        localFileMissing: 'The local file is gone',
        upload: 'The upload failed',
        directoryUnwritable: 'The export folder could not be created',
        writeFailed: 'The file could not be written',
        asset: 'The asset could not be read',
        probeFailed: 'The server could not be checked',
        signInFailed: 'Sign in failed',
        sessionFailed: 'The saved session could not be restored',
        capacityFailed: 'The storage quota could not be read',
        creation: 'The folder could not be created',
        emptyName: 'The folder name is empty',
    },
    {
        notSignedIn: '尚未登录',
        sessionExpired: '登录已过期, 请重新登录',
        unreachable: '无法连接服务器',
        rejected: '服务器拒绝了请求',
        protocol: '服务器返回了无法识别的响应',
        notCloudreve: '这个地址不是 Cloudreve 服务器',
        unauthorized: '该账号未获授权',
        credentialsRejected: '邮箱或密码错误',
        codeRejected: '验证码错误',
        serverAddressInvalid: '服务器地址格式不正确',
        serverMissing: '请先选择服务器',
        signInMissing: '请重新登录后再试',
        folderGone: '这个文件夹在服务器上已不存在',
        assetGone: '这个文件在服务器上已不存在',
        permission: '没有权限访问这个文件',
        permissionDenied: '设备媒体库需要权限',
        nameRejected: '服务器不接受这个名称',
        parentGone: '上级文件夹已不存在',
        alreadyExists: '同名文件夹已存在',
        quotaExceeded: '存储空间不足',
        sourceGone: '原文件在服务器上已不存在',
        destinationGone: '目标文件夹已不存在',
        conflict: '目标位置已有同名文件',
        sourceUnreadable: '无法读取本机文件',
        chunkFailed: '上传中断',
        interrupted: '上传已停止',
        sessionExpiredUpload: '上传会话已过期',
        unavailable: '无法打开本地索引',
        queryFailed: '本地索引查询失败',
        decodeFailed: '无法读取已存储的数据',
        encodeFailed: '无法写入数据',
        platform: '平台服务失败',
        decodePreviewFailed: '无法生成缩略图',
        serverAddressEmpty: '服务器地址不能为空',
        catalog: '本地索引失败',
        albumMissing: '这个相册已不存在',
        assetMissing: '这个资产已不存在',
        localAlbumMissing: '这个本机相册已不存在',
        notAFolder: '只能把文件夹添加为相册',
        listing: '无法列出这个文件夹',
        metadata: '无法保存相册信息',
        move: '无法移动这些文件',
        moveMismatch: '服务器移动的文件数量与请求不一致',
        localMedia: '设备媒体库操作失败',
        thumb: '无法加载缩略图',
        media: '无法加载这个文件',
        transport: '请求失败',
        directoryUnreadable: '无法读取这个本机文件夹',
        queueFailed: '无法把文件加入上传队列',
        taskMissing: '这个上传任务已不存在',
        taskNotFailed: '这个上传任务不处于失败状态',
        localFileMissing: '本机文件已不存在',
        upload: '上传失败',
        directoryUnwritable: '无法创建导出目录',
        writeFailed: '无法写入文件',
        asset: '无法读取这个资产',
        probeFailed: '无法检查服务器',
        signInFailed: '登录失败',
        sessionFailed: '无法恢复已保存的登录',
        capacityFailed: '无法读取存储配额',
        creation: '无法创建文件夹',
        emptyName: '文件夹名不能为空',
    },
);

/** The sentence a user sees for any error a command sent back. */
export function errorText(raw: unknown): string {
    if (
        !raw ||
        typeof raw !== 'object' ||
        !('kind' in raw) ||
        typeof (raw as {kind: unknown}).kind !== 'string'
    ) {
        return String(raw);
    }
    return describe(raw as AppError);
}

function describe(error: AppError): string {
    const t = i18n.pick(words);
    switch (error.kind) {
        case 'notSignedIn':
            return t.notSignedIn;
        case 'sessionExpired':
            return 'message' in error && error.message ? error.message : t.sessionExpired;
        case 'unreachable':
            return `${t.unreachable}: ${error.cause}`;
        case 'rejected':
            return `${t.rejected} (${error.status}): ${error.message}`;
        case 'protocol':
            return `${t.protocol}: ${error.cause}`;
        case 'notCloudreve':
            return `${t.notCloudreve}: ${error.detail}`;
        case 'unauthorized':
            return t.unauthorized;
        case 'credentialsRejected':
            return error.message ?? t.credentialsRejected;
        case 'codeRejected':
            return error.message ?? t.codeRejected;
        case 'serverAddressInvalid':
            return t.serverAddressInvalid;
        case 'serverMissing':
            return t.serverMissing;
        case 'signInMissing':
            return t.signInMissing;
        case 'folderGone':
            return `${t.folderGone}: ${error.uri}`;
        case 'assetGone':
            if ('asset' in error) return `${t.assetGone}: ${error.asset}`;
            if ('uri' in error) return `${t.assetGone}: ${error.uri}`;
            return t.assetGone;
        case 'permission':
            return 'uri' in error ? `${t.permission}: ${error.uri}` : t.permission;
        case 'permissionDenied':
            return t.permissionDenied;
        case 'nameRejected':
            return t.nameRejected;
        case 'parentGone':
            return `${t.parentGone}: ${error.path}`;
        case 'alreadyExists':
            return `${t.alreadyExists}: ${error.path}`;
        case 'quotaExceeded':
            return t.quotaExceeded;
        case 'sourceGone':
            return `${t.sourceGone}: ${error.uri}`;
        case 'destinationGone':
            return `${t.destinationGone}: ${error.uri}`;
        case 'conflict':
            return 'path' in error
                ? `${t.conflict}: ${error.path}`
                : `${t.conflict}: ${error.uri}`;
        case 'sourceUnreadable':
            return `${t.sourceUnreadable}: ${error.cause}`;
        case 'chunkFailed':
            return `${t.chunkFailed}: ${error.cause}`;
        case 'interrupted':
            return t.interrupted;
        case 'unavailable':
            return `${t.unavailable}: ${error.cause}`;
        case 'queryFailed':
            return `${t.queryFailed}: ${error.cause}`;
        case 'decodeFailed':
            return 'cause' in error ? `${t.decodeFailed}: ${error.cause}` : t.decodePreviewFailed;
        case 'encodeFailed':
            return `${t.encodeFailed}: ${error.cause}`;
        case 'platform':
            return `${t.platform}: ${error.cause}`;
        case 'catalog':
            return `${t.catalog}: ${describe(error.cause)}`;
        case 'listing':
            return 'album' in error
                ? `${t.listing}: ${error.album}: ${describe(error.cause)}`
                : `${t.listing}: ${error.uri}: ${describe(error.cause)}`;
        case 'metadata':
            return `${t.metadata}: ${describe(error.cause)}`;
        case 'move':
            return `${t.move}: ${describe(error.cause)}`;
        case 'moveMismatch':
            return t.moveMismatch;
        case 'localMedia':
            return `${t.localMedia}: ${describe(error.cause)}`;
        case 'thumb':
            return `${t.thumb}: ${describe(error.cause)}`;
        case 'media':
            return `${t.media}: ${describe(error.cause)}`;
        case 'transport':
            return `${t.transport}: ${describe(error.cause)}`;
        case 'directoryUnreadable':
            return `${t.directoryUnreadable}: ${error.uri}: ${describe(error.cause)}`;
        case 'queueFailed':
            return `${t.queueFailed}: ${describe(error.cause)}`;
        case 'taskMissing':
            return `${t.taskMissing}: ${error.task}`;
        case 'taskNotFailed':
            return `${t.taskNotFailed}: ${error.task}`;
        case 'localFileMissing':
            return `${t.localFileMissing}: ${error.asset}`;
        case 'upload':
            return `${t.upload}: ${describe(error.cause)}`;
        case 'directoryUnwritable':
            return `${t.directoryUnwritable}: ${error.cause}`;
        case 'writeFailed':
            return `${t.writeFailed}: ${error.path}: ${error.cause}`;
        case 'asset':
            return `${t.asset}: ${describe(error.cause)}`;
        case 'probeFailed':
            return `${t.probeFailed}: ${describe(error.cause)}`;
        case 'signInFailed':
            return `${t.signInFailed}: ${describe(error.cause)}`;
        case 'sessionFailed':
            return `${t.sessionFailed}: ${describe(error.cause)}`;
        case 'capacityFailed':
            return `${t.capacityFailed}: ${describe(error.cause)}`;
        case 'creation':
            return `${t.creation}: ${describe(error.cause)}`;
        case 'emptyName':
            return t.emptyName;
        case 'albumMissing':
            return `${t.albumMissing}: ${error.album}`;
        case 'assetMissing':
            return `${t.assetMissing}: ${error.asset}`;
        case 'localAlbumMissing':
            return `${t.localAlbumMissing}: ${error.uri}`;
        case 'notAFolder':
            return t.notAFolder;
        case 'store':
            return `${t.catalog}: ${describe(error.cause)}`;
    }
}

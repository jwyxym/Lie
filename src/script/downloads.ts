import { convertFileSrc, invoke } from '@tauri-apps/api/core';

export interface Download {
	id : number;
	animeId : string;
	episodeId : string;
	animeTitle : string;
	episodeTitle : string;
	coverUrl : string;
	source : string;
	pageUrl : string;
	status : 'queued' | 'downloading' | 'merging' | 'completed' | 'failed' | 'deleting';
	downloadedBytes : number;
	totalBytes : number | null;
	completedSegments : number;
	totalSegments : number;
	error : string;
}

export interface DownloadRequest {
	pageUrl : string;
	animeTitle : string;
	episodeTitle : string;
	coverUrl? : string;
}

export interface DownloadHeader {
	seriesId : string;
	title : string;
	busy : boolean;
	remove? : () => Promise<void>;
}

export const listDownloads = () : Promise<Download[]> => invoke<Download[]>('list_downloads');
export const startDownload = (request : DownloadRequest) : Promise<number> => {
	const normalized : DownloadRequest & { coverUrl : string; } = { ...request, coverUrl: request.coverUrl ?? '' };
	return invoke<number>('start_download', { request: normalized });
};
export const deleteDownload = (id : number) : Promise<void> => invoke<void>('delete_download', { id });
export const downloadDirectory = () : Promise<string> => invoke<string>('download_directory');
export const cachedVideo = async (pageUrl : string, coverUrl? : string) : Promise<string | undefined> => {
	const path : string | null = await invoke<string | null>('cached_video', { pageUrl, coverUrl });
	return path ? convertFileSrc(path) : undefined;
};

export const readCachedVideo = async (pageUrl : string, signal : AbortSignal) : Promise<Blob> => {
	signal.throwIfAborted();
	const total : number = await invoke<number>('cached_video_size', { pageUrl });
	if (!Number.isSafeInteger(total) || total <= 0) throw new Error('缓存视频大小无效');
	const parts : Blob[] = [];
	try {
		let offset : number = 0;
		while (offset < total) {
			signal.throwIfAborted();
			const bytes : ArrayBuffer = await invoke<ArrayBuffer>('cached_video_bytes', { pageUrl, offset });
			signal.throwIfAborted();
			if (bytes.byteLength !== Math.min(200 * 1024 * 1024, total - offset)) throw new Error('缓存视频读取不完整');
			// Keep immutable Blob parts rather than retaining every ArrayBuffer in the JS heap.
			parts.push(new Blob([bytes]));
			offset += bytes.byteLength;
		}
		return new Blob(parts, { type : 'video/mp4' });
	} finally {
		parts.length = 0;
	}
};

export const formatBytes = (bytes : number) : string => {
	if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
	if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
	return `${(bytes / 1024).toFixed(0)} KB`;
};

export const downloadStatus = (item : Download) : string => {
	if (item.status === 'queued') return '等待下载';
	if (item.status === 'merging') return '正在合并完整视频';
	if (item.status === 'completed') return `已缓存 · ${formatBytes(item.downloadedBytes)}`;
	if (item.status === 'failed') return `缓存失败：${item.error}`;
	if (item.status === 'deleting') return '正在停止并删除';
	const progress : string = item.totalSegments > 0
		? `${item.completedSegments}/${item.totalSegments} 个分片`
		: item.totalBytes ? `${Math.min(99, Math.floor(item.downloadedBytes / item.totalBytes * 100))}%` : '正在下载';
	return `${progress} · ${formatBytes(item.downloadedBytes)}`;
};

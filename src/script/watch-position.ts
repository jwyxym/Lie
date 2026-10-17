import { invoke } from '@tauri-apps/api/core';
import { onBeforeUnmount, onMounted, type Ref } from 'vue';
import { Snackbar } from '@varlet/ui';

// Finish earlier saves before restoring a newly opened episode.
let pending : Promise<void> = Promise.resolve();

export const getWatchPosition = async (pageUrl : string) : Promise<number> => {
	await pending.catch(() : void => {});
	return invoke<number>('get_watch_position', { pageUrl });
};

export const saveWatchPosition = (pageUrl : string, position : number, duration : number) : Promise<void> => {
	pending = pending.catch(() : void => {}).then(() : Promise<void> => invoke<void>('save_watch_position', { pageUrl, position, duration }));
	return pending;
};

export const useWatchPosition = (container : Ref<HTMLDivElement | null>, pageUrl : string) : void => {
	let media : HTMLVideoElement | null = null;
	let disposed : boolean = false;
	let loaded : boolean = false;
	let restored : boolean = false;
	let resumePosition : number = 0;
	let restoreTarget : number | undefined;
	let lastQueuedPosition : number | undefined;
	let lastSavedAt : number = 0;
	let hasActivity : boolean = false;
	let reportedSaveError : boolean = false;

	const restore = () : void => {
		if (!loaded || restored || !media || media.readyState < 1 || !Number.isFinite(media.duration) || media.duration <= 0) return;
		const target : number = Math.min(resumePosition, Math.max(0, media.duration - 1));
		if (target > 0) {
			try {
				restoreTarget = target;
				media.currentTime = target;
			} catch { restoreTarget = undefined; return; }
		}
		restored = true;
		lastQueuedPosition = target;
	};
	const save = (force : boolean = false, ended : boolean = false) : void => {
		if (!media || !restored || restoreTarget !== undefined || (media.seeking && !force)) return;
		const duration : number = media.duration;
		const position : number = ended || media.ended ? 0 : media.currentTime;
		if (!Number.isFinite(duration) || duration <= 0 || !Number.isFinite(position) || position < 0) return;
		if (!hasActivity && !ended) {
			if (Math.abs(position - (lastQueuedPosition ?? 0)) < 0.05) return;
			hasActivity = true;
		}
		if (!force && Date.now() - lastSavedAt < 5000) return;
		if (lastQueuedPosition !== undefined && Math.abs(position - lastQueuedPosition) < 0.05 && !ended) return;
		lastQueuedPosition = position;
		lastSavedAt = Date.now();
		void saveWatchPosition(pageUrl, position, duration).catch((error : unknown) : void => {
			if (lastQueuedPosition === position) lastQueuedPosition = undefined;
			console.error('Save watch position failed:', error);
			if (!disposed && !reportedSaveError) { reportedSaveError = true; Snackbar.error(`观看位置保存失败：${String(error)}`); }
		});
	};
	const finishRestore = () : boolean => {
		if (restoreTarget === undefined) return true;
		if (!media || media.seeking || Math.abs(media.currentTime - restoreTarget) >= 1) return false;
		restoreTarget = undefined;
		return true;
	};
	const update = () : void => {
		restore();
		if (!finishRestore()) return;
		if (media && media.currentTime > 0) hasActivity = true;
		save();
	};
	const pause = () : void => { if (finishRestore()) save(true); };
	const seeked = () : void => {
		if (restoreTarget !== undefined) { finishRestore(); return; }
		hasActivity = true;
		save(true);
	};
	const ended = () : void => { restoreTarget = undefined; hasActivity = true; save(true, true); };
	const background = () : void => { if (document.visibilityState === 'hidden') pause(); };
	const events : Record<string, EventListener> = {
		loadedmetadata: restore, durationchange: restore, canplay: restore, progress: restore,
		timeupdate: update, pause, seeked, ended
	};
	onMounted(async () : Promise<void> => {
		media = container.value?.querySelector('video') ?? null;
		if (!media) return;
		for (const [name, handler] of Object.entries(events)) media.addEventListener(name, handler);
		document.addEventListener('visibilitychange', background);
		window.addEventListener('pagehide', pause);
		try {
			const position : number = await getWatchPosition(pageUrl);
			if (disposed) return;
			resumePosition = Number.isFinite(position) && position > 0 ? position : 0;
		} catch (error : unknown) {
			if (disposed) return;
			Snackbar.error(`读取观看位置失败：${String(error)}`);
		}
		loaded = true;
		restore();
	});
	onBeforeUnmount(() : void => {
		pause();
		disposed = true;
		for (const [name, handler] of Object.entries(events)) media?.removeEventListener(name, handler);
		document.removeEventListener('visibilitychange', background);
		window.removeEventListener('pagehide', pause);
		media = null;
	});
};

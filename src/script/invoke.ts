import { invoke } from '@tauri-apps/api/core';

export interface Item  {
	name : string;
	url : string;
	img : string;
	count ?: string;
	desc ?: string;
};

export type Items = Array<Item>;

export interface Schedule {
	[key : number] : Items
};

export type Anthology = Array<Array<{ name : string; url : string; }>>;

export interface AniInfo {
	name : string;
	img : string;
	desc : string;
	date : string;
	links : Anthology;
};

export async function get_ani (url : string) : Promise<AniInfo | undefined> {
	try {
		const result = await invoke<[string, string, string, string, Anthology]>('get_ani', { url })
		return {
			name : result[0],
			desc : result[1],
			date : result[2],
			img : result[3],
			links : result[4]
		};
	} catch (e) {
		return undefined;
	}
};

export async function get_schedule () : Promise<Schedule> {
	try {
		return await invoke<Schedule>('get_schedule');
	} catch (e) {
		const result : Schedule = {};
		for (let i = 0; i < 7; i ++)
			result[i] = [];
		return result;
	}
};

export async function get_browse (year : number, page : number) : Promise<Items> {
	try {
		return JSON.parse(await invoke<string>('get_browse', { year, page }))
			.list
			.map((i : {
				vod_pic : string;
				vod_name : string;
				vod_blurb : string;
				url : string;
			}) => {
				return {
					name : i.vod_name,
					url : "https://anime.xifanacg.com" + i.url,
					img : i.vod_pic,
					desc : i.vod_blurb
				};
			});
	} catch (e) {
		return [];
	}
};

export async function get_video (url : string) : Promise<string> {
	try {
		console.log('get_video')
		const video = await invoke<ArrayBuffer>('get_video', { url });
		console.log('get_video')
		const view = new DataView(video);
		const buffer = new Uint8Array(video, 1);
		const head = view.getUint8(0);
		console.log(head)
		if (head === 1) {
			const text = new TextDecoder('utf-8').decode(buffer);
			return text;
		} else if (head === 2) {
			const blob = new Blob([buffer], { type: 'video/mp4' })
			return URL.createObjectURL(blob);
		}
		return '';
	} catch (e) {
		return '';
	}
}

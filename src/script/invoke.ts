import { invoke } from '@tauri-apps/api/core';
import { Snackbar } from '@varlet/ui';

export interface Item  {
	name : string;
	url : string;
	img : string;
	count ?: string;
	desc ?: string;
};

export type Items = Array<Item>;

export interface BrowseResult {
	list : Items;
	hasMore : boolean;
};

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
			date : result[1],
			desc : result[2],
			img : result[3],
			links : result[4]
		};
	} catch (e) {
		//@ts-ignore
		Snackbar['error'](e.toString());
		return undefined;
	}
};

export async function get_schedule () : Promise<Schedule> {
	try {
		return await invoke<Schedule>('get_schedule');
	} catch (e) {
		//@ts-ignore
		Snackbar['error'](e.toString());
		const result : Schedule = {};
		for (let i = 0; i < 7; i ++)
			result[i] = [];
		return result;
	}
};

export async function get_browse (status : number, year : number, page : number) : Promise<BrowseResult | undefined> {
	try {
		return await invoke<BrowseResult>('get_browse', { status, year, page });
	} catch (e) {
		//@ts-ignore
		Snackbar['error'](e.toString());
		return undefined;
	}
};

export async function get_video (url : string) : Promise<string> {
	try {
		return await invoke<string>('get_video', { url });
	} catch (e) {
		//@ts-ignore
		Snackbar['error'](e.toString());
		return '';
	}
}

export async function get_search (keyword : string, page : number = 1) : Promise<BrowseResult | undefined> {
	try {
		return await invoke<BrowseResult>('get_search', { keyword, page });
	} catch (e) {
		//@ts-ignore
		Snackbar['error'](e.toString());
		return undefined;
	}
}

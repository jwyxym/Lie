import { invoke } from '@tauri-apps/api/core';

export type Items = Array<{
	name : string;
	url : string;
	img : string;
	count ?: string;
	data ?: string
}>;

export interface Schedule {
	[key : number] : Items
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

export async function get_all () : Promise<Items> {
	try {
		return JSON.parse(await invoke<string>('get_all'))
			.itemListElement
			.map((i : {
				item : {
					datePublished : string;
					image : string;
					name : string;
					url : string;
				}
			}) => {
				return {
					name : i.item.name,
					url : i.item.url,
					img : i.item.image,
					data : i.item.datePublished
				};
			});
	} catch (e) {
		return [];
	}
}
import { openDB, type DBSchema } from 'idb';

interface LieDB extends DBSchema {
	videos: {
		key: string;
		value: {
			id: string;
			video: Blob;
			updatedAt: number;
		};
	};
}

const db = await openDB<LieDB>('lie-db', 1, {
	upgrade(db) {
		if (!db.objectStoreNames.contains('videos')) {
			db.createObjectStore('videos', { keyPath: 'id' });
		}
	}
});

export async function get(url: string): Promise<Blob | undefined> {
	const record = await db.get('videos', url);
	return record?.video;
}

export async function set(url: string, video: Blob): Promise<IDBValidKey> {
	return await db.put('videos', {
		id: url,
		video,
		updatedAt: Date.now()
	});
}

export async function del(url: string): Promise<void> {
	await db.delete('videos', url);
}

export async function clear(): Promise<void> {
	await db.clear('videos');
}
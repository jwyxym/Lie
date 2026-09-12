import { useRouter } from 'vue-router';

export function useTravel () {
	const router = useRouter();

	function to_ani (url : string, home : string, day : string, search : string) {
		router.push({
			name: 'animation',
			query: { url, home, day, search }
		});
	};

	function to_video (
		url : string,
		back : string,
		name : string,
		page : number,
		home : string,
		day : string,
		search : string
	) {
		router.push({
			name: 'video',
			query: { url, back, name, page, home, day, search }
		});
	};

	function to_home (home : string, day : string, search : string) {
		router.push({
			path: '/',
			query: { home, day, search }
		});
	};

	return {
		to_ani,
		to_video,
		to_home
	};
}

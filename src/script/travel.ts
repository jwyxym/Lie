import { useRouter, type Router } from 'vue-router';

export function useTravel () {
	const router : Router = useRouter();

	function to_ani (url : string, home : string, day : string, search : string) {
		router.push({
			name: 'animation',
			query: { url, home, day, search }
		});
	};

	const to_video = (
		url : string,
		back : string,
		name : string,
		page : number,
		home : string,
		day : string,
		search : string,
		cover : string = ''
	) : void => {
		router.push({
			name: 'video',
			query: { url, back, name, page, home, day, search, cover }
		});
	};

	function to_home (home : string, day : string, search : string) {
		router.push({
			path: '/',
			query: { home, day, search }
		});
	};

	function back () {
		router.back();
	};

	return {
		to_ani,
		to_video,
		to_home,
		back
	};
}

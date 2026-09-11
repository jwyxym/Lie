import { useRouter } from 'vue-router';

export function useTravel () {
	const router = useRouter();

	function to_ani (url : string) {
		router.push({
			name: 'animation',
			query: { url }
		});
	};

	function to_video (url : string, back : string, name : string, page : number) {
		router.push({
			name: 'video',
			query: { url, back, name, page }
		});
	};

	function to_home () {
		router.push('/');
	};

	return {
		to_ani,
		to_video,
		to_home
	};
}

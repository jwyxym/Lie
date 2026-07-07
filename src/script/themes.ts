import { Themes, StyleProvider } from '@varlet/ui';
import { reactive } from 'vue';

const themes = reactive({
	name : 'sun',
	change () {
		if (this.name === 'sun') {
			this.name = 'moon';
			StyleProvider(Themes.md3Dark);
		} else {
			this.name = 'sun';
			StyleProvider(Themes.md3Light);
		}
	}
});

export default themes;
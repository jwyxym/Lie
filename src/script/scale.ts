const resize = () => {
	const height = window.innerHeight;
	const width = window.innerWidth;
	document.documentElement.style.setProperty('--height', `${height}px`);
	document.documentElement.style.setProperty('--width', `${width}px`);
};

window.addEventListener('resize', resize);
resize();
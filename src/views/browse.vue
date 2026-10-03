<template>
	<div class = 'browse'>
		<var-cell :border = 'true' title = '状态'>
			<template #extra>
				<var-icon 
					:name = "browse.btn[0] ? 'chevron-down' : 'chevron-left'"
					:transition = '100'
					@click = '() => browse.btn[0] = !browse.btn[0]'
				/>
			</template>
		</var-cell>
		<var-collapse-transition :expand = 'browse.btn[0]'>
			<div>
				<var-button
					:type = "browse.status === 1 ? 'primary' : 'default'"
					@click = 'browse.select_status(1)'
				>
					连载中
				</var-button>
			</div>
			<div>
				<var-button
					:type = "browse.status === 2 ? 'primary' : 'default'"
					@click = 'browse.select_status(2)'
				>
					已完结
				</var-button>
			</div>
		</var-collapse-transition>
		<var-cell :border = 'true' title = '年份'>
			<template #extra>
				<var-icon 
					:name = "browse.btn[1] ? 'chevron-down' : 'chevron-left'"
					:transition = '100'
					@click = '() => browse.btn[1] = !browse.btn[1]'
				/>
			</template>
		</var-cell>
		<var-collapse-transition :expand = 'browse.btn[1]'>
			<div
				v-for = 'i in browse.years'
			>
				<var-button
					:type = "browse.year === i[1] ? 'primary' : 'default'"
					@click = 'browse.select_year(i[1])'
				>
					{{ i[0] }}
				</var-button>
			</div>
		</var-collapse-transition>
		<var-list
			:finished = 'browse.finished'
			v-model:loading = 'browse.loading'
			v-model:error = 'browse.error'
			@load = 'browse.load()'
			class = 'no-scrollbar'
			@scroll = 'browse.load_on'
		>
			<card
				v-for = 'i in browse.list'
				:item = 'i'
			/>
		</var-list>
	</div>
</template>
<script setup lang = 'ts'>
	import { onBeforeMount, reactive } from 'vue';
	import { get_browse, type Items } from '@/script/invoke';
	import card from '@/ui/card.vue';

	let request = 0;

	const browse = reactive({
		btn : [false, false],
		years : [] as Array<[string, number]>,
		status : 1,
		year : 0,
		ct : 1,
		select_year (year : number) {
			if (this.year === year)
				return;
			this.year = year;
			this.reset();
		},
		select_status (status : number) {
			if (this.status === status)
				return;
			this.status = status;
			this.reset();
		},
		reset () {
			request ++;
			this.ct = 1;
			this.finished = false;
			this.error = false;
			this.loaded = false;
			this.loading = false;
			this.list.length = 0;
			void this.load();
		},
		loaded : false,
		loading : false,
		finished : false,
		error : false,
		async load () {
			if (this.loaded || this.finished || this.error)
				return;
			this.loaded = true;
			this.loading = true;
			const id = ++ request;
			try {
				const result = await get_browse(this.status, this.year, this.ct);
				if (id !== request)
					return;
				if (!result) {
					this.error = true;
					return;
				}
				this.list.push(...result.list);
				this.ct ++;
				this.finished = !result.hasMore;
			} finally {
				if (id === request) {
					this.loading = false;
					this.loaded = false;
				}
			}
		},
		load_on (event : Event) {
			if (!browse.list.length)
				return;
			const { scrollTop, scrollHeight, clientHeight } = event.target as HTMLElement;
			if (scrollHeight / browse.list.length < scrollHeight - scrollTop - clientHeight)
				return;
			browse.load();
		},
		list : [] as Items
	});

	onBeforeMount(() => {
		browse.years.push(['全部', 0]);
		const year = new Date().getFullYear();
		for (let i = year; i >= 2007; i --)
			browse.years.push([i.toString(), i]);
		browse.years.push(['更早', -1]);
	});
</script>
<style scoped lang = 'scss'>
	.browse {
		color: var(--color-primary);
		> div {
			margin-top: 5px;
		}
		> div:first-of-type,
		> div:nth-of-type(3) {
			display: flex;
			gap: 20px;
			height: 30px;
		}
		.var-collapse-transition__content {
			display: flex;
			flex-wrap: wrap;
			gap: 5px;
			> * {
				width: 75px;
				height: 40px;
				display: flex;
				justify-content: center;
				align-items: center;
			}
		}
		.var-list {
			height: calc(100% - 100px);
			width: 100%;
			overflow-y: auto;
			display: flex;
			flex-direction: column;
			gap: 5px;
		}
	}
</style>

<template>
	<TransitionGroup
		class = 'search'
		name = 'move'
		tag = 'div'
	>
		<div
			class = 'input'
			key = '0'
		>
			<var-input
				v-model = 'page.keywords'
				placeholder = '搜索'
				variant = 'outlined'
				size = 'small'
				clearable
				@keydown = 'page.keydown($event)'
				@clear = 'page.clear_verify()'
			/>
			<var-chip
				plain
				@click = 'page.get_verify()'
			>
				搜索
			</var-chip>
		</div>
		<div
			v-if = 'page.verify_src'
			class = 'verify'
			key = '1'
		>
			<var-input
				v-model = 'page.verify'
				placeholder = '请输入图片中的验证码'
				variant = 'outlined'
				size = 'small'
				clearable
				@keyup.enter = 'page.search()'
			/>
			<var-chip
				plain
				@click = 'page.search()'
			>
				验证
			</var-chip>
			<img
				:src = 'page.verify_src'
				alt = '验证码，点击刷新'
				@click = 'page.get_verify()'
			/>
		</div>
		<div
			v-if = 'page.items.length'
			class = 'results no-scrollbar'
			:style = "{ '--top' : page.top }"
			key = '2'
		>
			<card
				v-for = 'item in page.items'
				:key = 'item.url'
				:item = 'item'
			/>
		</div>
	</TransitionGroup>
</template>
<script setup lang = 'ts'>
	import { computed, onBeforeUnmount, reactive } from 'vue';
	import { get_verify_image, verify_search, type Items } from '@/script/invoke';
	import Card from '@/ui/card.vue';

	const page = reactive({
		keywords : '',
		verify : '',
		verify_src : '',
		top : computed(() : string => {
			return (page.verify_src ? 180 : 60) + 'px';
		}),
		items : [] as Items,
		submitting : false,
		get_verify : function () {
			if (!page.keywords?.trim()) return;
			get_verify_image(page.keywords.trim())
				.then((i) => {
					if (i?.Image) {
						this.clear_verify();
						this.verify = '';
						this.verify_src = URL.createObjectURL(new Blob([
							new Uint8Array(i.Image.bytes)
						], { type: i.Image.mime }));
					} else if (i) {
						page.items.length = 0;
						this.clear_verify();
						setTimeout(() => {
							page.items = i.Data
								.map(i => {
									return {
										name : i[0],
										url : "https://anime.xifanacg.com" + i[1],
										img : i[2]
									}
								});
						}, 200);
					}
				});
		},
		search : function () {
			if (!page.keywords?.trim() || !page.verify?.trim()) return;
			verify_search(page.verify.trim(), page.keywords.trim())
				.then((i) => {
					this.clear_verify();
					console.log(i)
					setTimeout(() => {
						page.items = i
					}, 200);
				});
		},
		clear_verify : function () {
			if (this.verify_src)
				URL.revokeObjectURL(this.verify_src);
			this.verify_src = '';
		},
		keydown : function (event : KeyboardEvent) {
			if (event.key === 'Enter')
				this.get_verify();
			else
				this.clear_verify();
		}
	});

	onBeforeUnmount(() => page.clear_verify());
</script>
<style scoped lang = 'scss'>
	.search {
		width: 100%;
		height: 100%;
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		.input,
		.verify {
			height: 60px;
			width: 90%;
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 10px;
			.var-chip {
				width: 60px;
				height: 30px;
				&:hover {
					cursor: pointer;
				}
			}
			.var-input {
				width: calc(100% - 80px);
			}
			img {
				height: 50px;
			}
		}
		.results {
			position: absolute;
			top: 0;
  			transform: translateX(var(--move-x, 0)) translateY(var(--top));
			height: calc(100% - var(--top));
			background-color: antiquewhite;
			width: 100%;
			overflow-y: auto;
			display: flex;
			flex-direction: column;
			gap: 5px;
			transition: all 0.2s ease;
		}
	}
	.move {
		&-enter-active,
		&-leave-active {
			transition: transform 0.2s ease;
		}

		&-enter-from {
			--move-x: 100%;
		}

		&-leave-to {
			--move-x: -100%;
		}

		&-enter-to,
		&-leave-from {
			--move-x: 0;
		}
	}
</style>

<script lang="ts">
	import Icon from './Icon.svelte';

	let {
		open = $bindable(false),
		oncover,
		ondescription,
		onselect,
		onremove,
	}: {
		open?: boolean;
		oncover: () => void;
		ondescription: () => void;
		onselect: () => void;
		onremove: () => void;
	} = $props();

	const itemClass =
		'flex w-full items-center gap-3 rounded-xl px-3 py-3 text-left text-sm hover:bg-gray';
</script>

{#if open}
	<div class="fixed inset-0 z-40 bg-black/40" role="presentation">
		<button
			type="button"
			class="absolute inset-0 size-full cursor-default"
			aria-label="关闭菜单"
			onclick={() => (open = false)}
		></button>
		<div
			class="absolute inset-x-3 bottom-[calc(1rem+var(--safe-bottom))] rounded-2xl bg-light p-2 shadow-2xl"
			role="menu"
			tabindex="-1"
		>
			<button
				onclick={() => {
					open = false;
					oncover();
				}}
				class={itemClass}
			>
				<Icon name="image" size={20} /> 设置相册封面
			</button>
			<button
				onclick={() => {
					open = false;
					ondescription();
				}}
				class={itemClass}
			>
				<Icon name="info" size={20} /> 编辑描述
			</button>
			<button
				onclick={() => {
					open = false;
					onselect();
				}}
				class={itemClass}
			>
				<Icon name="download" size={20} /> 选择并下载
			</button>
			<button
				onclick={() => {
					open = false;
					onremove();
				}}
				class="flex w-full items-center gap-3 rounded-xl px-3 py-3 text-left text-sm text-red-500 hover:bg-gray"
			>
				<Icon name="trash" size={20} /> 移除相册
			</button>
		</div>
	</div>
{/if}

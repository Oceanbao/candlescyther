import { renderComponent, renderSnippet } from '$lib/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';
import { createRawSnippet } from 'svelte';
import DataTableActions from './data-table-action.svelte';
import DataTableSortButton from '$lib/components/data-table/data-table-sort-button.svelte';
import { Checkbox } from '$lib/components/ui/checkbox/index.js';
import type { TLogs } from '$lib/server/client';

// This type is used to define the shape of data.
// Columns are where you define the core of what your table will look like.
// They define the data that will be displayed, how it will be formatted, sorted and filtered.
// Use Zod schema for typing.

const maplevel = (level: number) => {
	switch (level) {
		case 0:
			return 'TRACE';
		case 1:
			return 'DEBUG';
		case 2:
			return 'INFO';
		case 3:
			return 'WARN';
		case 4:
			return 'ERROR';
		case 5:
			return 'FATAL';
	}
	return 'INFO';
};

export const columns: ColumnDef<TLogs>[] = [
	{
		accessorKey: 'log_level',
		header: () => {
			const headerSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">Level</div>`
			}));
			return renderSnippet(headerSnippet);
		},
		cell: ({ row }) => {
			// const formatter = new Intl.NumberFormat('en-US', {
			// 	style: 'currency',
			// 	currency: 'USD'
			// });

			const cellSnippet = createRawSnippet<[{ log_level: string }]>((getData) => {
				const { log_level } = getData();
				return {
					render: () => `<div class="text-right font-medium">${log_level}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				log_level: maplevel(row.original.log_level)
			});
		}
	},
	{
		accessorKey: 'log_target',
		header: () => {
			const headerSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">Target</div>`
			}));
			return renderSnippet(headerSnippet);
		},
		cell: ({ row }) => {
			const cellSnippet = createRawSnippet<[{ log_target: string }]>((getData) => {
				const { log_target } = getData();
				return {
					render: () => `<div class="text-right font-medium">${log_target}</div>`
				};
			});

			return renderSnippet(cellSnippet, {
				log_target: row.original.log_target
			});
		}
	},
	{
		accessorKey: 'log_message',
		header: () => {
			const amountHeaderSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-center">Message</div>`
			}));
			return renderSnippet(amountHeaderSnippet);
		},
		cell: ({ row }) => {
			const payloadCellSnippet = createRawSnippet<[{ log_message: string }]>((getData) => {
				const { log_message } = getData();
				return {
					render: () => `<div class="text-right font-medium">${JSON.stringify(log_message)}</div>`
				};
			});

			return renderSnippet(payloadCellSnippet, {
				log_message: row.original.log_message as string
			});
		}
	},
	{
		accessorKey: 'log_timestamp',
		header: ({ column }) =>
			renderComponent(DataTableSortButton, {
				style: 'text-align: center; width: 100%; height: 100%;',
				onclick: column.getToggleSortingHandler()
			}),
		cell: ({ row }) => {
			const createdCellSnippet = createRawSnippet<[{ log_timestamp: string }]>((getData) => {
				const { log_timestamp } = getData();
				return {
					render: () => `<div class="text-center">${log_timestamp}</div>`
				};
			});

			return renderSnippet(createdCellSnippet, {
				log_timestamp: row.original.log_timestamp
			});
		}
	},
	{
		id: 'actions',
		// You can access the row data using row.original in the cell function.
		// Use this to handle actions for your row eg. use the id to make a DELETE call to your API.
		cell: ({ row }) => {
			// You can pass whatever you need from `row.original` to the component
			return renderComponent(DataTableActions, { id: `${row.original.id}` });
		}
	},
	{
		id: 'select',
		header: ({ table }) =>
			renderComponent(Checkbox, {
				checked: table.getIsAllPageRowsSelected(),
				indeterminate: table.getIsSomePageRowsSelected() && !table.getIsAllPageRowsSelected(),
				onCheckedChange: (value) => table.toggleAllPageRowsSelected(!!value),
				'aria-label': 'Select all'
			}),
		cell: ({ row }) =>
			renderComponent(Checkbox, {
				checked: row.getIsSelected(),
				onCheckedChange: (value) => row.toggleSelected(!!value),
				'aria-label': 'Select row'
			}),
		enableSorting: false,
		enableHiding: false
	}
];

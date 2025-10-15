import { renderComponent, renderSnippet } from '$lib/components/ui/data-table';
import type { ColumnDef } from '@tanstack/table-core';
import { createRawSnippet } from 'svelte';
import DataTableActions from './data-table-actions.svelte';
import DataTableEmailButton from './data-table-email-button.svelte';
import { Checkbox } from '$lib/components/ui/checkbox/index.js';

// This type is used to define the shape of data.
// Columns are where you define the core of what your table will look like.
// They define the data that will be displayed, how it will be formatted, sorted and filtered.
// Use Zod schema for typing.
export type Payment = {
	id: string;
	amount: number;
	status: 'pending' | 'processing' | 'success' | 'failed';
	email: string;
};

export const columns: ColumnDef<Payment>[] = [
	{
		accessorKey: 'amount',
		header: () => {
			const amountHeaderSnippet = createRawSnippet(() => ({
				render: () => `<div class="text-right">Amount</div>`
			}));
			return renderSnippet(amountHeaderSnippet);
		},
		cell: ({ row }) => {
			const formatter = new Intl.NumberFormat('en-US', {
				style: 'currency',
				currency: 'USD'
			});

			const amountCellSnippet = createRawSnippet<[{ amount: number }]>((getAmount) => {
				const { amount } = getAmount();
				const formatted = formatter.format(amount);
				return {
					render: () => `<div class="text-right font-medium">${formatted}</div>`
				};
			});

			return renderSnippet(amountCellSnippet, {
				amount: row.original.amount
			});
		}
	},
	{
		accessorKey: 'email',
		header: ({ column }) =>
			renderComponent(DataTableEmailButton, {
				onclick: column.getToggleSortingHandler()
			})
	},
	{
		id: 'actions',
		// You can access the row data using row.original in the cell function.
		// Use this to handle actions for your row eg. use the id to make a DELETE call to your API.
		cell: ({ row }) => {
			// You can pass whatever you need from `row.original` to the component
			return renderComponent(DataTableActions, { id: row.original.id });
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

export async function getPayment() {
	const statuses: ('pending' | 'processing' | 'success' | 'failed')[] = [
		'pending',
		'processing',
		'success',
		'failed'
	];

	const domains = [
		'gmail.com',
		'yahoo.com',
		'hotmail.com',
		'outlook.com',
		'company.com',
		'business.org',
		'mail.net',
		'email.io'
	];

	// const getRandomId = (): string => {
	// 	return (
	// 		Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15)
	// 	);
	// };

	const getRandomAmount = (): number => {
		return parseFloat((Math.random() * 1000).toFixed(2));
	};

	const getRandomStatus = (): Payment['status'] => {
		return statuses[Math.floor(Math.random() * statuses.length)];
	};

	const getRandomEmail = (): string => {
		const username = Math.random().toString(36).substring(2, 10);
		const domain = domains[Math.floor(Math.random() * domains.length)];
		return `${username}@${domain}`;
	};

	const payments: Payment[] = [];

	for (let i = 0; i < 50; i++) {
		payments.push({
			id: `${(i + 2) * 3}`,
			amount: getRandomAmount(),
			status: getRandomStatus(),
			email: getRandomEmail()
		});
	}

	return Promise.resolve(payments);
}

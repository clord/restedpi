import { useCallback, useEffect } from '/js/depend/react/';
import { h } from '/js/html.js';
import { useAppStore } from '/js/hooks/useApp.js';
import { Table } from '/js/Table/';

function DeviceStatus({ cell }) {
  switch (cell.value) {
    case 'ok':
      return h('span', {}, 'OK');
    case 'connected':
      return h('span', {}, 'Connected');
    case 'disconnected':
      return h('span', {}, 'Disconnected');
  }
  return null;
}

function ActionCol({ cell }) {
  const url = cell.value;
  const handleRemove = useCallback(() => {
    console.log('handle remove ', url);
  }, [url]);

  const handleEdit = useCallback(() => {
    console.log('handle edit ', url);
  }, [url]);

  return [
    h(
      'button',
      { className: 'text-sm text-gray-500 py-1 px-3', onClick: handleRemove },
      'Remove'
    ),
    h(
      'button',
      {
        className:
          'text-sm bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-3 rounded',
        onClick: handleEdit,
      },
      'Edit'
    ),
  ];
}

function AddDevice({ cell }) {
  const handleAdd = useCallback(() => {
    console.log('handle add ');
  }, []);
  return h(
    'a',
    {
      className:
        'bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded',
      href: '/devices/available',
    },
    'Add Device'
  );
}

function DevicesConfiguredTable({ data }) {
  return h(Table, {
    columns: [
      { Header: 'Name', accessor: 'device.name' },
      {
        Header: 'Status',
        accessor: 'device.status',
        Cell: DeviceStatus,
        style: { textAlign: 'center' },
      },
      { Header: 'Description', accessor: 'device.description' },
      {
        accessor: 'url',
        Cell: ActionCol,
        style: { textAlign: 'right' },
      },
    ],
    data,
  });
}

export function DevicesConfigured() {
  useEffect(
    useAppStore(x => x.devices.read),
    []
  );
  const configured = useAppStore(x => x.devices.configured);
  const data = [...configured].map(([key, device]) => ({
    device,
    key,
    url: key,
  }));

  return h('article', { className: 'max-w-sm w-full lg:max-w-full' }, [
    h('div', { className: 'flex mb-4 justify-between items-baseline' }, [
      h(
        'h1',
        { className: 'text-gray-900 font-bold text-xl mb-3' },
        'Configured Devices'
      ),
      h(AddDevice, {}),
    ]),
    h(DevicesConfiguredTable, { data }),
  ]);
}

import { h } from '/js/html.js';
import { Link } from '/js/depend/wouter/';
import { useAppStore } from '/js/hooks/useApp.js';
import { useCallback, useEffect } from '/react/';
import { Table } from '/js/Table/';

function UnitValue({ row }) {
  const { unit, value } = row.original;
  switch (unit) {
    case 'DegC':
      return value.toFixed(1) + '℃';
    case 'KPa':
      return value.toFixed(1) + ' kPa';
    case 'Boolean':
      return value == 1 ? 'On' : 'Off';
  }
  return value;
}

function ActionCol() {
  return h(
    Link,
    {
      className:
        'mx-1 bg-blue-300 hover:bg-blue-700 text-white font-bold py-1 px-3 rounded',
      key: 'history',
      to: `/`,
    },
    'History'
  );
}

function AllSensorsTable({ data }) {
  return h(Table, {
    columns: [
      { Header: 'Name', accessor: 'name' },
      {
        Header: 'Status',
        accessor: 'status',
      },
      {
        Header: 'Sensor Value',
        className: 'text-right',
        id: 'sensor',
        sortType: (rowA, rowB, columnId, desc) => {
          const unitComp = rowA.original.unit.localeCompare(rowB.original.unit);
          if (unitComp !== 0) {
            return unitComp;
          }
          return rowA.original.value - rowB.original.value;
        },
        accessor: ({ unit, value }) => `${value} ${unit}`,
        Cell: UnitValue,
      },
      {
        accessor: 'slug',
        Cell: ActionCol,
        className: 'text-xs',
        columnStyle: { width: '110px' },
      },
    ],
    data,
  });
}

export function Sensors(props) {
  const sensors = useAppStore(x => x.sensors.all);
  const data = Object.keys(sensors).flatMap(name =>
    sensors[name].map((x, i) => ({ ...x, name: i > 0 ? `${name}/${i}` : name }))
  );

  return h('article', { className: 'max-w-sm w-full lg:max-w-full' }, [
    h(
      'div',
      { className: 'flex mb-4 justify-between items-baseline', key: 0 },
      [
        h(
          'h1',
          { className: 'text-gray-900 font-bold text-xl mb-3', key: 0 },
          'All Sensor Values'
        ),
      ]
    ),
    h(AllSensorsTable, { data, key: 1 }),
  ]);
}

export { Sensors as default };

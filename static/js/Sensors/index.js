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

function Sparkline({ points }) {
  let mx = -99999999,
    mn = 9999999,
    x = 0,
    pointsStr = '';

  for (const y of points) {
    mx = Math.max(mx, y);
    mn = Math.min(mn, y);
    x = x + 1;
    pointsStr = pointsStr + ` ${x},${y}`;
  }

  const height = mx - mn;
  const width = x - 1;
  const offset = height + mn * 2; // flip co-ordinates and move back into frame
  const transform = `matrix(1 0 0 -1 0 ${offset})`;
  const viewBox = `0 ${mn} ${width} ${height}`;

  return h(
    'svg',
    {
      version: '1.1',
      viewBox,
      preserveAspectRatio: 'none',
      baseProfile: 'full',
      width: '100px',
      height: '25px',
    },
    h('polyline', {
      points: pointsStr,
      stroke: 'rgba(0,0,0,1)',
      strokeWidth: '.8%',
      fill: 'none',
      transform,
    })
  );
}

function ActionCol() {
  return h('div', { className: 'flex items-center' }, [
    h(Sparkline, {
      key: '1',
      points: [
        20,
        15,
        20,
        31,
        37,
        7,
        31,
        37,
        32,
        33,
        35,
        7,
        32,
        39,
        31,
        31,
        35,
        20,
        31,
        37,
        15,
        20,
        7,
        33,
        39,
        32,
        31,
        35,
        7,
        33,
        37,
        31,
        35,
        34,
        33,
        35,
        7,
        32,
        39,
        33,
        33,
        35,
        20,
        15,
        20,
        29,
      ],
    }),
    h(
      Link,
      {
        key: '2',
        className:
          'mx-1 bg-blue-400 hover:bg-blue-700 text-white font-bold py-1 px-3 rounded',
        key: 'history',
        to: `/`,
      },
      'History'
    ),
  ]);
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
    sensors[name].map((x, i) => ({ ...x, name: `${name}/${i}` }))
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

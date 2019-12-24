import React from '/js/depend/react/';

function _extends() {
  _extends = Object.assign || function (target) {
    for (var i = 1; i < arguments.length; i++) {
      var source = arguments[i];

      for (var key in source) {
        if (Object.prototype.hasOwnProperty.call(source, key)) {
          target[key] = source[key];
        }
      }
    }

    return target;
  };

  return _extends.apply(this, arguments);
}

function _objectWithoutPropertiesLoose(source, excluded) {
  if (source == null) return {};
  var target = {};
  var sourceKeys = Object.keys(source);
  var key, i;

  for (i = 0; i < sourceKeys.length; i++) {
    key = sourceKeys[i];
    if (excluded.indexOf(key) >= 0) continue;
    target[key] = source[key];
  }

  return target;
}

function _toPrimitive(input, hint) {
  if (typeof input !== "object" || input === null) return input;
  var prim = input[Symbol.toPrimitive];

  if (prim !== undefined) {
    var res = prim.call(input, hint || "default");
    if (typeof res !== "object") return res;
    throw new TypeError("@@toPrimitive must return a primitive value.");
  }

  return (hint === "string" ? String : Number)(input);
}

function _toPropertyKey(arg) {
  var key = _toPrimitive(arg, "string");

  return typeof key === "symbol" ? key : String(key);
}

var renderErr = 'Renderer Error';
var actions = {
  init: 'init'
};
var defaultColumn = {
  Cell: function Cell(_ref) {
    var _ref$cell$value = _ref.cell.value,
        value = _ref$cell$value === void 0 ? '' : _ref$cell$value;
    return value;
  },
  width: 150,
  minWidth: 0,
  maxWidth: Number.MAX_SAFE_INTEGER
};
function defaultOrderByFn(arr, funcs, dirs) {
  return [].concat(arr).sort(function (rowA, rowB) {
    for (var i = 0; i < funcs.length; i += 1) {
      var sortFn = funcs[i];
      var desc = dirs[i] === false || dirs[i] === 'desc';
      var sortInt = sortFn(rowA, rowB);

      if (sortInt !== 0) {
        return desc ? -sortInt : sortInt;
      }
    }

    return dirs[0] ? rowA.index - rowB.index : rowB.index - rowA.index;
  });
}
function defaultGroupByFn(rows, columnId) {
  return rows.reduce(function (prev, row, i) {
    // TODO: Might want to implement a key serializer here so
    // irregular column values can still be grouped if needed?
    var resKey = "" + row.values[columnId];
    prev[resKey] = Array.isArray(prev[resKey]) ? prev[resKey] : [];
    prev[resKey].push(row);
    return prev;
  }, {});
}

function mergeProps() {
  for (var _len = arguments.length, propList = new Array(_len), _key = 0; _key < _len; _key++) {
    propList[_key] = arguments[_key];
  }

  return propList.reduce(function (props, next) {
    var style = next.style,
        className = next.className,
        rest = _objectWithoutPropertiesLoose(next, ["style", "className"]);

    props = _extends({}, props, {}, rest);

    if (style) {
      props.style = props.style ? _extends({}, props.style || {}, {}, style || {}) : style;
    }

    if (className) {
      props.className = props.className ? props.className + ' ' + className : className;
    }

    if (props.className === '') {
      delete props.className;
    }

    return props;
  }, {});
}

function handlePropGetter(prevProps, userProps, meta) {
  // Handle a lambda, pass it the previous props
  if (typeof userProps === 'function') {
    return handlePropGetter({}, userProps(prevProps, meta));
  } // Handle an array, merge each item as separate props


  if (Array.isArray(userProps)) {
    return mergeProps.apply(void 0, [prevProps].concat(userProps));
  } // Handle an object by default, merge the two objects


  return mergeProps(prevProps, userProps);
}

var makePropGetter = function makePropGetter(hooks, meta) {
  if (meta === void 0) {
    meta = {};
  }

  return function (userProps) {
    if (userProps === void 0) {
      userProps = {};
    }

    return [].concat(hooks, [userProps]).reduce(function (prev, next) {
      return handlePropGetter(prev, next, _extends({}, meta, {
        userProps: userProps
      }));
    }, {});
  };
};
var reduceHooks = function reduceHooks(hooks, initial, meta) {
  if (meta === void 0) {
    meta = {};
  }

  return hooks.reduce(function (prev, next) {
    var nextValue = next(prev, meta);

    if (window.env.NODE_ENV !== 'production') {
      if (typeof nextValue === 'undefined') {
        console.info(next);
        throw new Error('React Table: A reducer hook ☝️ just returned undefined! This is not allowed.');
      }
    }

    return nextValue;
  }, initial);
};
var loopHooks = function loopHooks(hooks, meta) {
  if (meta === void 0) {
    meta = {};
  }

  return hooks.forEach(function (hook) {
    var nextValue = hook(meta);

    if (window.env.NODE_ENV !== 'production') {
      if (typeof nextValue !== 'undefined') {
        console.info(hook, nextValue);
        throw new Error('React Table: A loop-type hook ☝️ just returned a value! This is not allowed.');
      }
    }
  });
};
function ensurePluginOrder(plugins, befores, pluginName, afters) {
  var pluginIndex = plugins.findIndex(function (plugin) {
    return plugin.pluginName === pluginName;
  });

  if (pluginIndex === -1) {
    if (window.env.NODE_ENV !== 'production') {
      throw new Error("The plugin \"" + pluginName + "\" was not found in the plugin list!\nThis usually means you need to need to name your plugin hook by setting the 'pluginName' property of the hook function, eg:\n\n  " + pluginName + ".pluginName = '" + pluginName + "'\n");
    }
  }

  befores.forEach(function (before) {
    var beforeIndex = plugins.findIndex(function (plugin) {
      return plugin.pluginName === before;
    });

    if (beforeIndex > -1 && beforeIndex > pluginIndex) {
      if (window.env.NODE_ENV !== 'production') {
        throw new Error("React Table: The " + pluginName + " plugin hook must be placed after the " + before + " plugin hook!");
      }
    }
  });
  afters.forEach(function (after) {
    var afterIndex = plugins.findIndex(function (plugin) {
      return plugin.pluginName === after;
    });

    if (window.env.NODE_ENV !== 'production') {
      if (afterIndex > -1 && afterIndex < pluginIndex) {
        throw new Error("React Table: The " + pluginName + " plugin hook must be placed before the " + after + " plugin hook!");
      }
    }
  });
}
function functionalUpdate(updater, old) {
  return typeof updater === 'function' ? updater(old) : updater;
}
function useGetLatest(obj) {
  var ref = React.useRef();
  ref.current = obj;
  return React.useCallback(function () {
    return ref.current;
  }, []);
} // SSR has issues with useLayoutEffect still, so use useEffect during SSR

var safeUseLayoutEffect = typeof document !== 'undefined' ? React.useLayoutEffect : React.useEffect;
function useMountedLayoutEffect(fn, deps) {
  var mountedRef = React.useRef(false);
  safeUseLayoutEffect(function () {
    if (mountedRef.current) {
      fn();
    }

    mountedRef.current = true; // eslint-disable-next-line
  }, deps);
}
function useAsyncDebounce(defaultFn, defaultWait) {
  if (defaultWait === void 0) {
    defaultWait = 0;
  }

  var debounceRef = React.useRef({});
  debounceRef.current.defaultFn = defaultFn;
  debounceRef.current.defaultWait = defaultWait;
  var debounce = React.useCallback(function _callee2(fn, wait) {
    return regeneratorRuntime.async(function _callee2$(_context2) {
      while (1) {
        switch (_context2.prev = _context2.next) {
          case 0:
            if (fn === void 0) {
              fn = debounceRef.current.defaultFn;
            }

            if (wait === void 0) {
              wait = debounceRef.current.defaultWait;
            }

            if (!debounceRef.current.promise) {
              debounceRef.current.promise = new Promise(function (resolve, reject) {
                debounceRef.current.resolve = resolve;
                debounceRef.current.reject = reject;
              });
            }

            if (debounceRef.current.timeout) {
              clearTimeout(debounceRef.current.timeout);
            }

            debounceRef.current.timeout = setTimeout(function _callee() {
              return regeneratorRuntime.async(function _callee$(_context) {
                while (1) {
                  switch (_context.prev = _context.next) {
                    case 0:
                      delete debounceRef.current.timeout;
                      _context.prev = 1;
                      _context.t0 = debounceRef.current;
                      _context.next = 5;
                      return regeneratorRuntime.awrap(fn());

                    case 5:
                      _context.t1 = _context.sent;

                      _context.t0.resolve.call(_context.t0, _context.t1);

                      _context.next = 12;
                      break;

                    case 9:
                      _context.prev = 9;
                      _context.t2 = _context["catch"](1);
                      debounceRef.current.reject(_context.t2);

                    case 12:
                      _context.prev = 12;
                      delete debounceRef.current.promise;
                      return _context.finish(12);

                    case 15:
                    case "end":
                      return _context.stop();
                  }
                }
              }, null, null, [[1, 9, 12, 15]]);
            }, wait);
            return _context2.abrupt("return", debounceRef.current.promise);

          case 6:
          case "end":
            return _context2.stop();
        }
      }
    });
  }, []);
  return debounce;
}
function useConsumeHookGetter(hooks, hookName) {
  var getter = useGetLatest(hooks[hookName]);
  hooks[hookName] = undefined;
  return getter;
}
function makeRenderer(instance, column, meta) {
  if (meta === void 0) {
    meta = {};
  }

  return function (type, userProps) {
    if (userProps === void 0) {
      userProps = {};
    }

    var Comp = typeof type === 'string' ? column[type] : type;

    if (typeof Comp === 'undefined') {
      throw new Error(renderErr);
    }

    return flexRender(Comp, _extends({}, instance, {
      column: column
    }, meta, {}, userProps));
  };
}
function flexRender(Comp, props) {
  return isReactComponent(Comp) ? React.createElement(Comp, props) : Comp;
}

function isClassComponent(component) {
  return typeof component === 'function' && !!function () {
    var proto = Object.getPrototypeOf(component);
    return proto.prototype && proto.prototype.isReactComponent;
  }();
}

function isFunctionComponent(component) {
  return typeof component === 'function';
}

function isExoticComponent(component) {
  return typeof component === 'object' && typeof component.$$typeof === 'symbol' && ['react.memo', 'react.forward_ref'].includes(component.$$typeof.description);
}

function isReactComponent(component) {
  return isClassComponent(component) || isFunctionComponent(component) || isExoticComponent(component);
}

function decorateColumn(column, userDefaultColumn, parent, depth, index) {
  // Apply the userDefaultColumn
  column = _extends({}, defaultColumn, {}, userDefaultColumn, {}, column); // First check for string accessor

  var _column = column,
      id = _column.id,
      accessor = _column.accessor,
      Header = _column.Header;

  if (typeof accessor === 'string') {
    id = id || accessor;
    var accessorPath = accessor.split('.');

    accessor = function accessor(row) {
      return getBy(row, accessorPath);
    };
  }

  if (!id && typeof Header === 'string' && Header) {
    id = Header;
  }

  if (!id && column.columns) {
    console.error(column);
    throw new Error('A column ID (or unique "Header" value) is required!');
  }

  if (!id) {
    console.error(column);
    throw new Error('A column ID (or string accessor) is required!');
  }

  column = _extends({
    // Make sure there is a fallback header, just in case
    Header: function Header() {
      return React.createElement(React.Fragment, null, "\xA0");
    },
    Footer: function Footer() {
      return React.createElement(React.Fragment, null, "\xA0");
    }
  }, column, {
    // Materialize and override this stuff
    id: id,
    accessor: accessor,
    parent: parent,
    depth: depth,
    index: index
  });
  return column;
} // Build the visible columns, headers and flat column list


function decorateColumnTree(columns, defaultColumn, parent, depth) {
  if (depth === void 0) {
    depth = 0;
  }

  return columns.map(function (column, columnIndex) {
    column = decorateColumn(column, defaultColumn, parent, depth, columnIndex);

    if (column.columns) {
      column.columns = decorateColumnTree(column.columns, defaultColumn, column, depth + 1);
    }

    return column;
  });
} // Build the header groups from the bottom up

function makeHeaderGroups(flatColumns, defaultColumn) {
  var headerGroups = []; // Build each header group from the bottom up

  var buildGroup = function buildGroup(columns, depth) {
    var headerGroup = {
      headers: []
    };
    var parentColumns = []; // Do any of these columns have parents?

    var hasParents = columns.some(function (col) {
      return col.parent;
    });
    columns.forEach(function (column) {
      // Are we the first column in this group?
      var isFirst = !parentColumns.length; // What is the latest (last) parent column?

      var latestParentColumn = [].concat(parentColumns).reverse()[0]; // If the column has a parent, add it if necessary

      if (column.parent) {
        var similarParentColumns = parentColumns.filter(function (d) {
          return d.originalId === column.parent.id;
        });

        if (isFirst || latestParentColumn.originalId !== column.parent.id) {
          parentColumns.push(_extends({}, column.parent, {
            originalId: column.parent.id,
            id: [column.parent.id, similarParentColumns.length].join('_')
          }));
        }
      } else if (hasParents) {
        // If other columns have parents, we'll need to add a place holder if necessary
        var originalId = [column.id, 'placeholder'].join('_');

        var _similarParentColumns = parentColumns.filter(function (d) {
          return d.originalId === originalId;
        });

        var placeholderColumn = decorateColumn({
          originalId: originalId,
          id: [column.id, 'placeholder', _similarParentColumns.length].join('_'),
          placeholderOf: column
        }, defaultColumn);

        if (isFirst || latestParentColumn.originalId !== placeholderColumn.originalId) {
          parentColumns.push(placeholderColumn);
        }
      } // Establish the new headers[] relationship on the parent


      if (column.parent || hasParents) {
        latestParentColumn = [].concat(parentColumns).reverse()[0];
        latestParentColumn.headers = latestParentColumn.headers || [];

        if (!latestParentColumn.headers.includes(column)) {
          latestParentColumn.headers.push(column);
        }
      }

      column.totalHeaderCount = column.headers ? column.headers.reduce(function (sum, header) {
        return sum + header.totalHeaderCount;
      }, 0) : 1; // Leaf node columns take up at least one count

      headerGroup.headers.push(column);
    });
    headerGroups.push(headerGroup);

    if (parentColumns.length) {
      buildGroup(parentColumns);
    }
  };

  buildGroup(flatColumns);
  return headerGroups.reverse();
}
var pathObjCache = new Map();
function getBy(obj, path, def) {
  if (!path) {
    return obj;
  }

  var cacheKey = typeof path === 'function' ? path : JSON.stringify(path);

  var pathObj = pathObjCache.get(cacheKey) || function () {
    var pathObj = makePathArray(path);
    pathObjCache.set(cacheKey, pathObj);
    return pathObj;
  }();

  var val;

  try {
    val = pathObj.reduce(function (cursor, pathPart) {
      return cursor[pathPart];
    }, obj);
  } catch (e) {// continue regardless of error
  }

  return typeof val !== 'undefined' ? val : def;
}
function getFirstDefined() {
  for (var _len = arguments.length, args = new Array(_len), _key = 0; _key < _len; _key++) {
    args[_key] = arguments[_key];
  }

  for (var i = 0; i < args.length; i += 1) {
    if (typeof args[i] !== 'undefined') {
      return args[i];
    }
  }
}
function isFunction(a) {
  if (typeof a === 'function') {
    return a;
  }
}
function flattenBy(columns, childKey) {
  var flatColumns = [];

  var recurse = function recurse(columns) {
    columns.forEach(function (d) {
      if (!d[childKey]) {
        flatColumns.push(d);
      } else {
        recurse(d[childKey]);
      }
    });
  };

  recurse(columns);
  return flatColumns;
}
function expandRows(rows, _ref) {
  var manualExpandedKey = _ref.manualExpandedKey,
      expanded = _ref.expanded,
      _ref$expandSubRows = _ref.expandSubRows,
      expandSubRows = _ref$expandSubRows === void 0 ? true : _ref$expandSubRows;
  var expandedRows = [];

  var handleRow = function handleRow(row) {
    row.isExpanded = row.original && row.original[manualExpandedKey] || expanded[row.id];
    row.canExpand = row.subRows && !!row.subRows.length;
    expandedRows.push(row);

    if (expandSubRows && row.subRows && row.subRows.length && row.isExpanded) {
      row.subRows.forEach(handleRow);
    }
  };

  rows.forEach(handleRow);
  return expandedRows;
}
function getFilterMethod(filter, userFilterTypes, filterTypes) {
  return isFunction(filter) || userFilterTypes[filter] || filterTypes[filter] || filterTypes.text;
}
function shouldAutoRemoveFilter(autoRemove, value) {
  return autoRemove ? autoRemove(value) : typeof value === 'undefined';
} //

var reOpenBracket = /\[/g;
var reCloseBracket = /\]/g;

function makePathArray(obj) {
  return flattenDeep(obj) // remove all periods in parts
  .map(function (d) {
    return String(d).replace('.', '_');
  }) // join parts using period
  .join('.') // replace brackets with periods
  .replace(reOpenBracket, '.').replace(reCloseBracket, '') // split it back out on periods
  .split('.');
}

function flattenDeep(arr, newArr) {
  if (newArr === void 0) {
    newArr = [];
  }

  if (!Array.isArray(arr)) {
    newArr.push(arr);
  } else {
    for (var i = 0; i < arr.length; i += 1) {
      flattenDeep(arr[i], newArr);
    }
  }

  return newArr;
}

var defaultCells = function defaultCells(cell) {
  return cell.filter(function (d) {
    return d.column.isVisible;
  });
};

var defaultGetHeaderProps = function defaultGetHeaderProps(props, _ref) {
  var column = _ref.column;
  return _extends({
    key: "header_" + column.id,
    colSpan: column.totalVisibleHeaderCount
  }, props);
};

var defaultGetFooterProps = function defaultGetFooterProps(props, _ref2) {
  var column = _ref2.column;
  return _extends({
    key: "footer_" + column.id,
    colSpan: column.totalVisibleHeaderCount
  }, props);
};

var defaultGetHeaderGroupProps = function defaultGetHeaderGroupProps(props, _ref3) {
  var index = _ref3.index;
  return _extends({
    key: "headerGroup_" + index
  }, props);
};

var defaultGetFooterGroupProps = function defaultGetFooterGroupProps(props, _ref4) {
  var index = _ref4.index;
  return _extends({
    key: "footerGroup_" + index
  }, props);
};

var defaultGetRowProps = function defaultGetRowProps(props, _ref5) {
  var row = _ref5.row;
  return _extends({
    key: "row_" + row.id
  }, props);
};

var defaultGetCellProps = function defaultGetCellProps(props, _ref6) {
  var cell = _ref6.cell;
  return _extends({}, props, {
    key: "cell_" + cell.row.id + "_" + cell.column.id
  });
};

function makeDefaultPluginHooks() {
  return {
    useOptions: [],
    stateReducers: [],
    useControlledState: [],
    columns: [],
    columnsDeps: [],
    flatColumns: [],
    flatColumnsDeps: [],
    headerGroups: [],
    headerGroupsDeps: [],
    useInstanceBeforeDimensions: [],
    useInstance: [],
    useRows: [],
    cells: [defaultCells],
    prepareRow: [],
    getTableProps: [],
    getTableBodyProps: [],
    getHeaderGroupProps: [defaultGetHeaderGroupProps],
    getFooterGroupProps: [defaultGetFooterGroupProps],
    getHeaderProps: [defaultGetHeaderProps],
    getFooterProps: [defaultGetFooterProps],
    getRowProps: [defaultGetRowProps],
    getCellProps: [defaultGetCellProps],
    useFinalInstance: []
  };
}

actions.resetHiddenColumns = 'resetHiddenColumns';
actions.toggleHideColumn = 'toggleHideColumn';
actions.setHiddenColumns = 'setHiddenColumns';
actions.toggleHideAllColumns = 'toggleHideAllColumns';
var useColumnVisibility = function useColumnVisibility(hooks) {
  hooks.getToggleHiddenProps = [defaultGetToggleHiddenProps];
  hooks.getToggleHideAllColumnsProps = [defaultGetToggleHideAllColumnsProps];
  hooks.stateReducers.push(reducer);
  hooks.useInstanceBeforeDimensions.push(useInstanceBeforeDimensions);
  hooks.headerGroupsDeps.push(function (deps, _ref) {
    var instance = _ref.instance;
    return [].concat(deps, [instance.state.hiddenColumns]);
  });
  hooks.useInstance.push(useInstance);
};
useColumnVisibility.pluginName = 'useColumnVisibility';

var defaultGetToggleHiddenProps = function defaultGetToggleHiddenProps(props, _ref2) {
  var column = _ref2.column;
  return [props, {
    onChange: function onChange(e) {
      column.toggleHidden(!e.target.checked);
    },
    style: {
      cursor: 'pointer'
    },
    checked: column.isVisible,
    title: 'Toggle Column Visible'
  }];
};

var defaultGetToggleHideAllColumnsProps = function defaultGetToggleHideAllColumnsProps(props, _ref3) {
  var instance = _ref3.instance;
  return [props, {
    onChange: function onChange(e) {
      instance.toggleHideAllColumns(!e.target.checked);
    },
    style: {
      cursor: 'pointer'
    },
    checked: !instance.allColumnsHidden && !instance.state.hiddenColumns.length,
    title: 'Toggle All Columns Hidden',
    indeterminate: !instance.allColumnsHidden && instance.state.hiddenColumns.length
  }];
};

function reducer(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      hiddenColumns: []
    }, state);
  }

  if (action.type === actions.resetHiddenColumns) {
    return _extends({}, state, {
      hiddenColumns: instance.initialState.hiddenColumns || []
    });
  }

  if (action.type === actions.toggleHideColumn) {
    var should = typeof action.value !== 'undefined' ? action.value : !state.hiddenColumns.includes(action.columnId);
    var hiddenColumns = should ? [].concat(state.hiddenColumns, [action.columnId]) : state.hiddenColumns.filter(function (d) {
      return d !== action.columnId;
    });
    return _extends({}, state, {
      hiddenColumns: hiddenColumns
    });
  }

  if (action.type === actions.setHiddenColumns) {
    return _extends({}, state, {
      hiddenColumns: functionalUpdate(action.value, state.hiddenColumns)
    });
  }

  if (action.type === actions.toggleHideAllColumns) {
    var shouldAll = typeof action.value !== 'undefined' ? action.value : !state.hiddenColumns.length;
    return _extends({}, state, {
      hiddenColumns: shouldAll ? instance.flatColumns.map(function (d) {
        return d.id;
      }) : []
    });
  }
}

function useInstanceBeforeDimensions(instance) {
  var headers = instance.headers,
      hiddenColumns = instance.state.hiddenColumns;
  var isMountedRef = React.useRef(false);

  if (!isMountedRef.current) ;

  var handleColumn = function handleColumn(column, parentVisible) {
    column.isVisible = parentVisible && !hiddenColumns.includes(column.id);
    var totalVisibleHeaderCount = 0;

    if (column.headers && column.headers.length) {
      column.headers.forEach(function (subColumn) {
        return totalVisibleHeaderCount += handleColumn(subColumn, column.isVisible);
      });
    } else {
      totalVisibleHeaderCount = column.isVisible ? 1 : 0;
    }

    column.totalVisibleHeaderCount = totalVisibleHeaderCount;
    return totalVisibleHeaderCount;
  };

  var totalVisibleHeaderCount = 0;
  headers.forEach(function (subHeader) {
    return totalVisibleHeaderCount += handleColumn(subHeader, true);
  });
}

function useInstance(instance) {
  var flatHeaders = instance.flatHeaders,
      dispatch = instance.dispatch,
      flatColumns = instance.flatColumns,
      hiddenColumns = instance.state.hiddenColumns;
  var getInstance = useGetLatest(instance);
  var allColumnsHidden = flatColumns.length === hiddenColumns.length;
  var toggleHideColumn = React.useCallback(function (columnId, value) {
    return dispatch({
      type: actions.toggleHideColumn,
      columnId: columnId,
      value: value
    });
  }, [dispatch]);
  var setHiddenColumns = React.useCallback(function (value) {
    return dispatch({
      type: actions.setHiddenColumns,
      value: value
    });
  }, [dispatch]);
  var toggleHideAllColumns = React.useCallback(function (value) {
    return dispatch({
      type: actions.toggleHideAllColumns,
      value: value
    });
  }, [dispatch]); // Snapshot hook and disallow more from being added

  var getToggleHideAllColumnsPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getToggleHideAllColumnsProps');
  var getToggleHideAllColumnsProps = makePropGetter(getToggleHideAllColumnsPropsHooks(), {
    instance: getInstance()
  }); // Snapshot hook and disallow more from being added

  var getToggleHiddenPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getToggleHiddenProps');
  flatHeaders.forEach(function (column) {
    column.toggleHidden = function (value) {
      dispatch({
        type: actions.toggleHideColumn,
        columnId: column.id,
        value: value
      });
    };

    column.getToggleHiddenProps = makePropGetter(getToggleHiddenPropsHooks(), {
      instance: getInstance(),
      column: column
    });
  });
  Object.assign(instance, {
    allColumnsHidden: allColumnsHidden,
    toggleHideColumn: toggleHideColumn,
    setHiddenColumns: setHiddenColumns,
    toggleHideAllColumns: toggleHideAllColumns,
    getToggleHideAllColumnsProps: getToggleHideAllColumnsProps
  });
}

var defaultInitialState = {};
var defaultColumnInstance = {};

var defaultReducer = function defaultReducer(state, action, prevState) {
  return state;
};

var defaultGetSubRows = function defaultGetSubRows(row, index) {
  return row.subRows || [];
};

var defaultGetRowId = function defaultGetRowId(row, index, parent) {
  return "" + (parent ? [parent.id, index].join('.') : index);
};

var defaultUseControlledState = function defaultUseControlledState(d) {
  return d;
};

function applyDefaults(props) {
  var _props$initialState = props.initialState,
      initialState = _props$initialState === void 0 ? defaultInitialState : _props$initialState,
      _props$defaultColumn = props.defaultColumn,
      defaultColumn = _props$defaultColumn === void 0 ? defaultColumnInstance : _props$defaultColumn,
      _props$getSubRows = props.getSubRows,
      getSubRows = _props$getSubRows === void 0 ? defaultGetSubRows : _props$getSubRows,
      _props$getRowId = props.getRowId,
      getRowId = _props$getRowId === void 0 ? defaultGetRowId : _props$getRowId,
      _props$stateReducer = props.stateReducer,
      stateReducer = _props$stateReducer === void 0 ? defaultReducer : _props$stateReducer,
      _props$useControlledS = props.useControlledState,
      useControlledState = _props$useControlledS === void 0 ? defaultUseControlledState : _props$useControlledS,
      rest = _objectWithoutPropertiesLoose(props, ["initialState", "defaultColumn", "getSubRows", "getRowId", "stateReducer", "useControlledState"]);

  return _extends({}, rest, {
    initialState: initialState,
    defaultColumn: defaultColumn,
    getSubRows: getSubRows,
    getRowId: getRowId,
    stateReducer: stateReducer,
    useControlledState: useControlledState
  });
}

var useTable = function useTable(props) {
  for (var _len = arguments.length, plugins = new Array(_len > 1 ? _len - 1 : 0), _key = 1; _key < _len; _key++) {
    plugins[_key - 1] = arguments[_key];
  }

  // Apply default props
  props = applyDefaults(props); // Add core plugins

  plugins = [useColumnVisibility].concat(plugins); // Create the table instance

  var instanceRef = React.useRef({}); // Create a getter for the instance (helps avoid a lot of potential memory leaks)

  var getInstance = useGetLatest(instanceRef.current); // Assign the props, plugins and hooks to the instance

  Object.assign(getInstance(), _extends({}, props, {
    plugins: plugins,
    hooks: makeDefaultPluginHooks()
  })); // Allow plugins to register hooks as early as possible

  plugins.filter(Boolean).forEach(function (plugin) {
    plugin(getInstance().hooks);
  });
  var getUseOptionsHooks = useConsumeHookGetter(getInstance().hooks, 'useOptions'); // Allow useOptions hooks to modify the options coming into the table

  Object.assign(getInstance(), reduceHooks(getUseOptionsHooks(), applyDefaults(props)));

  var _getInstance = getInstance(),
      data = _getInstance.data,
      userColumns = _getInstance.columns,
      initialState = _getInstance.initialState,
      defaultColumn = _getInstance.defaultColumn,
      getSubRows = _getInstance.getSubRows,
      getRowId = _getInstance.getRowId,
      stateReducer = _getInstance.stateReducer,
      useControlledState = _getInstance.useControlledState; // Snapshot hook and disallow more from being added


  var getStateReducers = useConsumeHookGetter(getInstance().hooks, 'stateReducers'); // Setup user reducer ref

  var getStateReducer = useGetLatest(stateReducer); // Build the reducer

  var reducer = React.useCallback(function (state, action) {
    // Detect invalid actions
    if (!action.type) {
      console.info({
        action: action
      });
      throw new Error('Unknown Action 👆');
    } // Reduce the state from all plugin reducers


    return [].concat(getStateReducers(), Array.isArray(getStateReducer()) ? getStateReducer() : [getStateReducer()]).reduce(function (s, handler) {
      return handler(s, action, state, getInstance()) || s;
    }, state);
  }, [getStateReducers, getStateReducer, getInstance]); // Start the reducer

  var _React$useReducer = React.useReducer(reducer, undefined, function () {
    return reducer(initialState, {
      type: actions.init
    });
  }),
      reducerState = _React$useReducer[0],
      dispatch = _React$useReducer[1]; // Snapshot hook and disallow more from being added


  var getUseControlledStateHooks = useConsumeHookGetter(getInstance().hooks, 'useControlledState'); // Allow the user to control the final state with hooks

  var state = reduceHooks([].concat(getUseControlledStateHooks(), [useControlledState]), reducerState, {
    instance: getInstance()
  });
  Object.assign(getInstance(), {
    state: state,
    dispatch: dispatch
  }); // Snapshot hook and disallow more from being added

  var getColumnsHooks = useConsumeHookGetter(getInstance().hooks, 'columns'); // Snapshot hook and disallow more from being added

  var getColumnsDepsHooks = useConsumeHookGetter(getInstance().hooks, 'columnsDeps'); // Decorate All the columns

  var columns = React.useMemo(function () {
    return decorateColumnTree(reduceHooks(getColumnsHooks(), userColumns, {
      instance: getInstance()
    }), defaultColumn);
  }, [defaultColumn, getColumnsHooks, getInstance, userColumns].concat(reduceHooks(getColumnsDepsHooks(), [], {
    instance: getInstance()
  })));
  getInstance().columns = columns; // Get the flat list of all columns and allow hooks to decorate
  // those columns (and trigger this memoization via deps)

  var flatColumns = React.useMemo(function () {
    return flattenBy(columns, 'columns');
  }, [columns]);
  getInstance().flatColumns = flatColumns; // Access the row model

  var _React$useMemo = React.useMemo(function () {
    var flatRows = []; // Access the row's data

    var accessRow = function accessRow(originalRow, i, depth, parent) {
      if (depth === void 0) {
        depth = 0;
      }

      // Keep the original reference around
      var original = originalRow;
      var id = getRowId(originalRow, i, parent);
      var row = {
        id: id,
        original: original,
        index: i,
        depth: depth,
        cells: [{}] // This is a dummy cell

      };
      flatRows.push(row); // Process any subRows

      var subRows = getSubRows(originalRow, i);

      if (subRows) {
        row.subRows = subRows.map(function (d, i) {
          return accessRow(d, i, depth + 1, row);
        });
      } // Override common array functions (and the dummy cell's getCellProps function)
      // to show an error if it is accessed without calling prepareRow


      var unpreparedAccessWarning = function unpreparedAccessWarning() {
        throw new Error('React-Table: You have not called prepareRow(row) one or more rows you are attempting to render.');
      };

      row.cells.map = unpreparedAccessWarning;
      row.cells.filter = unpreparedAccessWarning;
      row.cells.forEach = unpreparedAccessWarning;
      row.cells[0].getCellProps = unpreparedAccessWarning; // Create the cells and values

      row.values = {};
      flatColumns.forEach(function (_ref) {
        var id = _ref.id,
            accessor = _ref.accessor;
        row.values[id] = accessor ? accessor(originalRow, i, {
          subRows: subRows,
          depth: depth,
          data: data
        }) : undefined;
      });
      return row;
    }; // Use the resolved data


    var accessedData = data.map(function (d, i) {
      return accessRow(d, i);
    });
    return [accessedData, flatRows];
  }, [data, flatColumns, getRowId, getSubRows]),
      rows = _React$useMemo[0],
      flatRows = _React$useMemo[1];

  getInstance().rows = rows;
  getInstance().flatRows = flatRows; // Snapshot hook and disallow more from being added

  var flatColumnsHooks = useConsumeHookGetter(getInstance().hooks, 'flatColumns'); // Snapshot hook and disallow more from being added

  var flatColumnsDepsHooks = useConsumeHookGetter(getInstance().hooks, 'flatColumnsDeps'); // Get the flat list of all columns AFTER the rows
  // have been access, and allow hooks to decorate
  // those columns (and trigger this memoization via deps)

  flatColumns = React.useMemo(function () {
    return reduceHooks(flatColumnsHooks(), flatColumns, {
      instance: getInstance()
    });
  }, [flatColumns, flatColumnsHooks, getInstance].concat(reduceHooks(flatColumnsDepsHooks(), [], {
    instance: getInstance()
  })));
  getInstance().flatColumns = flatColumns; // Snapshot hook and disallow more from being added

  var getHeaderGroups = useConsumeHookGetter(getInstance().hooks, 'headerGroups'); // Snapshot hook and disallow more from being added

  var getHeaderGroupsDeps = useConsumeHookGetter(getInstance().hooks, 'headerGroupsDeps'); // Make the headerGroups

  var headerGroups = React.useMemo(function () {
    return reduceHooks(getHeaderGroups(), makeHeaderGroups(flatColumns, defaultColumn), getInstance());
  }, [defaultColumn, flatColumns, getHeaderGroups, getInstance].concat(reduceHooks(getHeaderGroupsDeps(), [], {
    instance: getInstance()
  })));
  getInstance().headerGroups = headerGroups;
  var headers = React.useMemo(function () {
    return headerGroups.length ? headerGroups[0].headers : [];
  }, [headerGroups]);
  getInstance().headers = headers; // Provide a flat header list for utilities

  getInstance().flatHeaders = headerGroups.reduce(function (all, headerGroup) {
    return [].concat(all, headerGroup.headers);
  }, []); // Snapshot hook and disallow more from being added

  var getUseInstanceBeforeDimensions = useConsumeHookGetter(getInstance().hooks, 'useInstanceBeforeDimensions');
  loopHooks(getUseInstanceBeforeDimensions(), getInstance()); // Header Visibility is needed by this point

  var _calculateHeaderWidth = calculateHeaderWidths(headers),
      totalColumnsMinWidth = _calculateHeaderWidth[0],
      totalColumnsWidth = _calculateHeaderWidth[1],
      totalColumnsMaxWidth = _calculateHeaderWidth[2];

  getInstance().totalColumnsMinWidth = totalColumnsMinWidth;
  getInstance().totalColumnsWidth = totalColumnsWidth;
  getInstance().totalColumnsMaxWidth = totalColumnsMaxWidth; // Snapshot hook and disallow more from being added

  var getUseInstance = useConsumeHookGetter(getInstance().hooks, 'useInstance');
  loopHooks(getUseInstance(), getInstance()); // Snapshot hook and disallow more from being added

  var getHeaderPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getHeaderProps'); // Snapshot hook and disallow more from being added

  var getFooterPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getFooterProps') // Each materialized header needs to be assigned a render function and other
  // prop getter properties here.
  ;
  [].concat(getInstance().flatHeaders, getInstance().flatColumns).forEach(function (column) {
    // Give columns/headers rendering power
    column.render = makeRenderer(getInstance(), column); // Give columns/headers a default getHeaderProps

    column.getHeaderProps = makePropGetter(getHeaderPropsHooks(), {
      instance: getInstance(),
      column: column
    }); // Give columns/headers a default getFooterProps

    column.getFooterProps = makePropGetter(getFooterPropsHooks(), {
      instance: getInstance(),
      column: column
    });
  }); // Snapshot hook and disallow more from being added

  var getHeaderGroupPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getHeaderGroupProps'); // Snapshot hook and disallow more from being added

  var getFooterGroupPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getFooterGroupProps');
  getInstance().headerGroups = getInstance().headerGroups.filter(function (headerGroup, i) {
    // Filter out any headers and headerGroups that don't have visible columns
    headerGroup.headers = headerGroup.headers.filter(function (column) {
      var recurse = function recurse(headers) {
        return headers.filter(function (column) {
          if (column.headers) {
            return recurse(column.headers);
          }

          return column.isVisible;
        }).length;
      };

      if (column.headers) {
        return recurse(column.headers);
      }

      return column.isVisible;
    }); // Give headerGroups getRowProps

    if (headerGroup.headers.length) {
      headerGroup.getHeaderGroupProps = makePropGetter(getHeaderGroupPropsHooks(), {
        instance: getInstance(),
        headerGroup: headerGroup,
        index: i
      });
      headerGroup.getFooterGroupProps = makePropGetter(getFooterGroupPropsHooks(), {
        instance: getInstance(),
        headerGroup: headerGroup,
        index: i
      });
      return true;
    }

    return false;
  });
  getInstance().footerGroups = [].concat(getInstance().headerGroups).reverse(); // Run the rows (this could be a dangerous hook with a ton of data)
  // Snapshot hook and disallow more from being added

  var getUseRowsHooks = useConsumeHookGetter(getInstance().hooks, 'useRows');
  getInstance().rows = reduceHooks(getUseRowsHooks(), getInstance().rows, {
    instance: getInstance()
  }); // The prepareRow function is absolutely necessary and MUST be called on
  // any rows the user wishes to be displayed.
  // Snapshot hook and disallow more from being added

  var getPrepareRowHooks = useConsumeHookGetter(getInstance().hooks, 'prepareRow'); // Snapshot hook and disallow more from being added

  var getRowPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getRowProps'); // Snapshot hook and disallow more from being added

  var getCellPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getCellProps'); // Snapshot hook and disallow more from being added

  var cellsHooks = useConsumeHookGetter(getInstance().hooks, 'cells');
  getInstance().prepareRow = React.useCallback(function (row) {
    row.getRowProps = makePropGetter(getRowPropsHooks(), {
      instance: getInstance(),
      row: row
    }); // Build the visible cells for each row

    row.allCells = flatColumns.map(function (column) {
      var cell = {
        column: column,
        row: row,
        value: row.values[column.id]
      }; // Give each cell a getCellProps base

      cell.getCellProps = makePropGetter(getCellPropsHooks(), {
        instance: getInstance(),
        cell: cell
      }); // Give each cell a renderer function (supports multiple renderers)

      cell.render = makeRenderer(getInstance(), column, {
        row: row,
        cell: cell
      });
      return cell;
    });
    row.cells = reduceHooks(cellsHooks(), row.allCells, {
      instance: getInstance()
    }); // need to apply any row specific hooks (useExpanded requires this)

    loopHooks(getPrepareRowHooks(), row, {
      instance: getInstance()
    });
  }, [getRowPropsHooks, getInstance, flatColumns, cellsHooks, getPrepareRowHooks, getCellPropsHooks]); // Snapshot hook and disallow more from being added

  var getTablePropsHooks = useConsumeHookGetter(getInstance().hooks, 'getTableProps');
  getInstance().getTableProps = makePropGetter(getTablePropsHooks(), {
    instance: getInstance()
  }); // Snapshot hook and disallow more from being added

  var getTableBodyPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getTableBodyProps');
  getInstance().getTableBodyProps = makePropGetter(getTableBodyPropsHooks(), {
    instance: getInstance()
  }); // Snapshot hook and disallow more from being added

  var getUseFinalInstanceHooks = useConsumeHookGetter(getInstance().hooks, 'useFinalInstance');
  loopHooks(getUseFinalInstanceHooks(), getInstance());
  return getInstance();
};

function calculateHeaderWidths(headers, left) {
  if (left === void 0) {
    left = 0;
  }

  var sumTotalMinWidth = 0;
  var sumTotalWidth = 0;
  var sumTotalMaxWidth = 0;
  headers.forEach(function (header) {
    var subHeaders = header.headers;
    header.totalLeft = left;

    if (subHeaders && subHeaders.length) {
      var _calculateHeaderWidth2 = calculateHeaderWidths(subHeaders, left),
          totalMinWidth = _calculateHeaderWidth2[0],
          totalWidth = _calculateHeaderWidth2[1],
          totalMaxWidth = _calculateHeaderWidth2[2];

      header.totalMinWidth = totalMinWidth;
      header.totalWidth = totalWidth;
      header.totalMaxWidth = totalMaxWidth;
    } else {
      header.totalMinWidth = header.minWidth;
      header.totalWidth = Math.min(Math.max(header.minWidth, header.width), header.maxWidth);
      header.totalMaxWidth = header.maxWidth;
    }

    if (header.isVisible) {
      left += header.totalWidth;
      sumTotalMinWidth += header.totalMinWidth;
      sumTotalWidth += header.totalWidth;
      sumTotalMaxWidth += header.totalMaxWidth;
    }
  });
  return [sumTotalMinWidth, sumTotalWidth, sumTotalMaxWidth];
}

actions.toggleExpanded = 'toggleExpanded';
actions.toggleAllExpanded = 'toggleAllExpanded';
actions.setExpanded = 'setExpanded';
actions.resetExpanded = 'resetExpanded';
var useExpanded = function useExpanded(hooks) {
  hooks.getExpandedToggleProps = [defaultGetExpandedToggleProps];
  hooks.stateReducers.push(reducer$1);
  hooks.useInstance.push(useInstance$1);
};
useExpanded.pluginName = 'useExpanded';

var defaultGetExpandedToggleProps = function defaultGetExpandedToggleProps(props, _ref) {
  var row = _ref.row;
  return [props, {
    onClick: function onClick(e) {
      e.persist();
      row.toggleExpanded();
    },
    style: {
      cursor: 'pointer'
    },
    title: 'Toggle Expanded'
  }];
}; // Reducer


function reducer$1(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      expanded: {}
    }, state);
  }

  if (action.type === actions.resetExpanded) {
    return _extends({}, state, {
      expanded: instance.initialState.expanded || {}
    });
  }

  if (action.type === actions.setExpanded) {
    return _extends({}, state, {
      expanded: functionalUpdate(action.expanded, state.expanded)
    });
  }

  if (action.type === actions.toggleExpanded) {
    var id = action.id,
        setExpanded = action.expanded;
    var exists = state.expanded[id];
    var shouldExist = typeof setExpanded !== 'undefined' ? setExpanded : !exists;

    if (!exists && shouldExist) {
      var _extends2;

      return _extends({}, state, {
        expanded: _extends({}, state.expanded, (_extends2 = {}, _extends2[id] = true, _extends2))
      });
    } else if (exists && !shouldExist) {
      var _state$expanded = state.expanded,
          _ = _state$expanded[id],
          rest = _objectWithoutPropertiesLoose(_state$expanded, [id].map(_toPropertyKey));

      return _extends({}, state, {
        expanded: rest
      });
    } else {
      return state;
    }
  }
}

function useInstance$1(instance) {
  var data = instance.data,
      rows = instance.rows,
      _instance$manualExpan = instance.manualExpandedKey,
      manualExpandedKey = _instance$manualExpan === void 0 ? 'expanded' : _instance$manualExpan,
      _instance$paginateExp = instance.paginateExpandedRows,
      paginateExpandedRows = _instance$paginateExp === void 0 ? true : _instance$paginateExp,
      _instance$expandSubRo = instance.expandSubRows,
      expandSubRows = _instance$expandSubRo === void 0 ? true : _instance$expandSubRo,
      hooks = instance.hooks,
      _instance$autoResetEx = instance.autoResetExpanded,
      autoResetExpanded = _instance$autoResetEx === void 0 ? true : _instance$autoResetEx,
      expanded = instance.state.expanded,
      dispatch = instance.dispatch;
  var getAutoResetExpanded = useGetLatest(autoResetExpanded); // Bypass any effects from firing when this changes

  useMountedLayoutEffect(function () {
    if (getAutoResetExpanded()) {
      dispatch({
        type: actions.resetExpanded
      });
    }
  }, [dispatch, data]);

  var toggleExpanded = function toggleExpanded(id, expanded) {
    dispatch({
      type: actions.toggleExpanded,
      id: id,
      expanded: expanded
    });
  }; // use reference to avoid memory leak in #1608


  var getInstance = useGetLatest(instance);
  var getExpandedTogglePropsHooks = useConsumeHookGetter(getInstance().hooks, 'getExpandedToggleProps');
  hooks.prepareRow.push(function (row) {
    row.toggleExpanded = function (set) {
      return instance.toggleExpanded(row.id, set);
    };

    row.getExpandedToggleProps = makePropGetter(getExpandedTogglePropsHooks(), {
      instance: getInstance(),
      row: row
    });
  });
  var expandedRows = React.useMemo(function () {
    if (paginateExpandedRows) {
      return expandRows(rows, {
        manualExpandedKey: manualExpandedKey,
        expanded: expanded,
        expandSubRows: expandSubRows
      });
    }

    return rows;
  }, [paginateExpandedRows, rows, manualExpandedKey, expanded, expandSubRows]);
  var expandedDepth = React.useMemo(function () {
    return findExpandedDepth(expanded);
  }, [expanded]);
  Object.assign(instance, {
    preExpandedRows: rows,
    expandedRows: expandedRows,
    rows: expandedRows,
    toggleExpanded: toggleExpanded,
    expandedDepth: expandedDepth
  });
}

function findExpandedDepth(expanded) {
  var maxDepth = 0;
  Object.keys(expanded).forEach(function (id) {
    var splitId = id.split('.');
    maxDepth = Math.max(maxDepth, splitId.length);
  });
  return maxDepth;
}

var text = function text(rows, ids, filterValue) {
  rows = rows.filter(function (row) {
    return ids.some(function (id) {
      var rowValue = row.values[id];
      return String(rowValue).toLowerCase().includes(String(filterValue).toLowerCase());
    });
  });
  return rows;
};

text.autoRemove = function (val) {
  return !val;
};

var exactText = function exactText(rows, ids, filterValue) {
  return rows.filter(function (row) {
    return ids.some(function (id) {
      var rowValue = row.values[id];
      return rowValue !== undefined ? String(rowValue).toLowerCase() === String(filterValue).toLowerCase() : true;
    });
  });
};

exactText.autoRemove = function (val) {
  return !val;
};

var exactTextCase = function exactTextCase(rows, ids, filterValue) {
  return rows.filter(function (row) {
    return ids.some(function (id) {
      var rowValue = row.values[id];
      return rowValue !== undefined ? String(rowValue) === String(filterValue) : true;
    });
  });
};

exactTextCase.autoRemove = function (val) {
  return !val;
};

var includes = function includes(rows, ids, filterValue) {
  return rows.filter(function (row) {
    return ids.some(function (id) {
      var rowValue = row.values[id];
      return filterValue.includes(rowValue);
    });
  });
};

includes.autoRemove = function (val) {
  return !val || !val.length;
};

var includesAll = function includesAll(rows, ids, filterValue) {
  return rows.filter(function (row) {
    return ids.some(function (id) {
      var rowValue = row.values[id];
      return rowValue && rowValue.length && filterValue.every(function (val) {
        return rowValue.includes(val);
      });
    });
  });
};

includesAll.autoRemove = function (val) {
  return !val || !val.length;
};

var exact = function exact(rows, ids, filterValue) {
  return rows.filter(function (row) {
    return ids.some(function (id) {
      var rowValue = row.values[id];
      return rowValue === filterValue;
    });
  });
};

exact.autoRemove = function (val) {
  return typeof val === 'undefined';
};

var equals = function equals(rows, ids, filterValue) {
  return rows.filter(function (row) {
    return ids.some(function (id) {
      var rowValue = row.values[id]; // eslint-disable-next-line eqeqeq

      return rowValue == filterValue;
    });
  });
};

equals.autoRemove = function (val) {
  return val == null;
};

var between = function between(rows, ids, filterValue) {
  var _ref = filterValue || [],
      min = _ref[0],
      max = _ref[1];

  min = typeof min === 'number' ? min : -Infinity;
  max = typeof max === 'number' ? max : Infinity;

  if (min > max) {
    var temp = min;
    min = max;
    max = temp;
  }

  return rows.filter(function (row) {
    return ids.some(function (id) {
      var rowValue = row.values[id];
      return rowValue >= min && rowValue <= max;
    });
  });
};

between.autoRemove = function (val) {
  return !val || typeof val[0] !== 'number' && typeof val[1] !== 'number';
};

var filterTypes = /*#__PURE__*/Object.freeze({
  __proto__: null,
  text: text,
  exactText: exactText,
  exactTextCase: exactTextCase,
  includes: includes,
  includesAll: includesAll,
  exact: exact,
  equals: equals,
  between: between
});

actions.resetFilters = 'resetFilters';
actions.setFilter = 'setFilter';
actions.setAllFilters = 'setAllFilters';
var useFilters = function useFilters(hooks) {
  hooks.stateReducers.push(reducer$2);
  hooks.useInstance.push(useInstance$2);
};
useFilters.pluginName = 'useFilters';

function reducer$2(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      filters: []
    }, state);
  }

  if (action.type === actions.resetFilters) {
    return _extends({}, state, {
      filters: instance.initialState.filters || []
    });
  }

  if (action.type === actions.setFilter) {
    var columnId = action.columnId,
        filterValue = action.filterValue;
    var flatColumns = instance.flatColumns,
        userFilterTypes = instance.userFilterTypes;
    var column = flatColumns.find(function (d) {
      return d.id === columnId;
    });

    if (!column) {
      throw new Error("React-Table: Could not find a column with id: " + columnId);
    }

    var filterMethod = getFilterMethod(column.filter, userFilterTypes || {}, filterTypes);
    var previousfilter = state.filters.find(function (d) {
      return d.id === columnId;
    });
    var newFilter = functionalUpdate(filterValue, previousfilter && previousfilter.value); //

    if (shouldAutoRemoveFilter(filterMethod.autoRemove, newFilter)) {
      return _extends({}, state, {
        filters: state.filters.filter(function (d) {
          return d.id !== columnId;
        })
      });
    }

    if (previousfilter) {
      return _extends({}, state, {
        filters: state.filters.map(function (d) {
          if (d.id === columnId) {
            return {
              id: columnId,
              value: newFilter
            };
          }

          return d;
        })
      });
    }

    return _extends({}, state, {
      filters: [].concat(state.filters, [{
        id: columnId,
        value: newFilter
      }])
    });
  }

  if (action.type === actions.setAllFilters) {
    var filters = action.filters;
    var _flatColumns = instance.flatColumns,
        _userFilterTypes = instance.filterTypes;
    return _extends({}, state, {
      // Filter out undefined values
      filters: functionalUpdate(filters, state.filters).filter(function (filter) {
        var column = _flatColumns.find(function (d) {
          return d.id === filter.id;
        });

        var filterMethod = getFilterMethod(column.filter, _userFilterTypes || {}, filterTypes);

        if (shouldAutoRemoveFilter(filterMethod.autoRemove, filter.value)) {
          return false;
        }

        return true;
      })
    });
  }
}

function useInstance$2(instance) {
  var data = instance.data,
      rows = instance.rows,
      flatRows = instance.flatRows,
      flatColumns = instance.flatColumns,
      userFilterTypes = instance.filterTypes,
      manualFilters = instance.manualFilters,
      _instance$defaultCanF = instance.defaultCanFilter,
      defaultCanFilter = _instance$defaultCanF === void 0 ? false : _instance$defaultCanF,
      disableFilters = instance.disableFilters,
      filters = instance.state.filters,
      dispatch = instance.dispatch,
      _instance$autoResetFi = instance.autoResetFilters,
      autoResetFilters = _instance$autoResetFi === void 0 ? true : _instance$autoResetFi;

  var setFilter = function setFilter(columnId, filterValue) {
    dispatch({
      type: actions.setFilter,
      columnId: columnId,
      filterValue: filterValue
    });
  };

  var setAllFilters = function setAllFilters(filters) {
    dispatch({
      type: actions.setAllFilters,
      filters: filters
    });
  };

  flatColumns.forEach(function (column) {
    var id = column.id,
        accessor = column.accessor,
        columnDefaultCanFilter = column.defaultCanFilter,
        columnDisableFilters = column.disableFilters; // Determine if a column is filterable

    column.canFilter = accessor ? getFirstDefined(columnDisableFilters === true ? false : undefined, disableFilters === true ? false : undefined, true) : getFirstDefined(columnDefaultCanFilter, defaultCanFilter, false); // Provide the column a way of updating the filter value

    column.setFilter = function (val) {
      return setFilter(column.id, val);
    }; // Provide the current filter value to the column for
    // convenience


    var found = filters.find(function (d) {
      return d.id === id;
    });
    column.filterValue = found && found.value;
  });

  var _React$useMemo = React.useMemo(function () {
    if (manualFilters || !filters.length) {
      return [rows, flatRows];
    }

    var filteredFlatRows = []; // Filters top level and nested rows

    var filterRows = function filterRows(rows, depth) {
      if (depth === void 0) {
        depth = 0;
      }

      var filteredRows = rows;
      filteredRows = filters.reduce(function (filteredSoFar, _ref) {
        var columnId = _ref.id,
            filterValue = _ref.value;
        // Find the filters column
        var column = flatColumns.find(function (d) {
          return d.id === columnId;
        });

        if (!column) {
          return filteredSoFar;
        }

        if (depth === 0) {
          column.preFilteredRows = filteredSoFar;
        }

        var filterMethod = getFilterMethod(column.filter, userFilterTypes || {}, filterTypes);

        if (!filterMethod) {
          console.warn("Could not find a valid 'column.filter' for column with the ID: " + column.id + ".");
          return filteredSoFar;
        } // Pass the rows, id, filterValue and column to the filterMethod
        // to get the filtered rows back


        column.filteredRows = filterMethod(filteredSoFar, [columnId], filterValue);
        return column.filteredRows;
      }, rows); // Apply the filter to any subRows
      // We technically could do this recursively in the above loop,
      // but that would severely hinder the API for the user, since they
      // would be required to do that recursion in some scenarios

      filteredRows = filteredRows.map(function (row) {
        filteredFlatRows.push(row);

        if (!row.subRows) {
          return row;
        }

        return _extends({}, row, {
          subRows: row.subRows && row.subRows.length > 0 ? filterRows(row.subRows, depth + 1) : row.subRows
        });
      });
      return filteredRows;
    };

    return [filterRows(rows), filteredFlatRows];
  }, [manualFilters, filters, rows, flatRows, flatColumns, userFilterTypes]),
      filteredRows = _React$useMemo[0],
      filteredFlatRows = _React$useMemo[1];

  React.useMemo(function () {
    // Now that each filtered column has it's partially filtered rows,
    // lets assign the final filtered rows to all of the other columns
    var nonFilteredColumns = flatColumns.filter(function (column) {
      return !filters.find(function (d) {
        return d.id === column.id;
      });
    }); // This essentially enables faceted filter options to be built easily
    // using every column's preFilteredRows value

    nonFilteredColumns.forEach(function (column) {
      column.preFilteredRows = filteredRows;
      column.filteredRows = filteredRows;
    });
  }, [filteredRows, filters, flatColumns]);
  var getAutoResetFilters = useGetLatest(autoResetFilters);
  useMountedLayoutEffect(function () {
    if (getAutoResetFilters()) {
      dispatch({
        type: actions.resetFilters
      });
    }
  }, [dispatch, manualFilters ? null : data]);
  Object.assign(instance, {
    preFilteredRows: rows,
    preFilteredFlatRows: flatRows,
    filteredRows: filteredRows,
    filteredFlatRows: filteredFlatRows,
    rows: filteredRows,
    flatRows: filteredFlatRows,
    setFilter: setFilter,
    setAllFilters: setAllFilters
  });
}

actions.resetGlobalFilter = 'resetGlobalFilter';
actions.setGlobalFilter = 'setGlobalFilter';
var useGlobalFilter = function useGlobalFilter(hooks) {
  hooks.stateReducers.push(reducer$3);
  hooks.useInstance.push(useInstance$3);
};
useGlobalFilter.pluginName = 'useGlobalFilter';

function reducer$3(state, action, previousState, instance) {
  if (action.type === actions.resetGlobalFilter) {
    return _extends({}, state, {
      globalFilter: instance.initialState.globalFilter || undefined
    });
  }

  if (action.type === actions.setGlobalFilter) {
    var filterValue = action.filterValue;
    var userFilterTypes = instance.userFilterTypes;
    var filterMethod = getFilterMethod(instance.globalFilter, userFilterTypes || {}, filterTypes);
    var newFilter = functionalUpdate(filterValue, state.globalFilter); //

    if (shouldAutoRemoveFilter(filterMethod.autoRemove, newFilter)) {
      var globalFilter = state.globalFilter,
          stateWithoutGlobalFilter = _objectWithoutPropertiesLoose(state, ["globalFilter"]);

      return stateWithoutGlobalFilter;
    }

    return _extends({}, state, {
      globalFilter: newFilter
    });
  }
}

function useInstance$3(instance) {
  var data = instance.data,
      rows = instance.rows,
      flatRows = instance.flatRows,
      flatColumns = instance.flatColumns,
      userFilterTypes = instance.filterTypes,
      globalFilter = instance.globalFilter,
      manualGlobalFilter = instance.manualGlobalFilter,
      globalFilterValue = instance.state.globalFilter,
      dispatch = instance.dispatch,
      _instance$autoResetGl = instance.autoResetGlobalFilters,
      autoResetGlobalFilters = _instance$autoResetGl === void 0 ? true : _instance$autoResetGl,
      plugins = instance.plugins;
  ensurePluginOrder(plugins, [], 'useGlobalFilter', ['useSortBy', 'useExpanded']);

  var setGlobalFilter = function setGlobalFilter(filterValue) {
    dispatch({
      type: actions.setGlobalFilter,
      filterValue: filterValue
    });
  }; // TODO: Create a filter cache for incremental high speed multi-filtering
  // This gets pretty complicated pretty fast, since you have to maintain a
  // cache for each row group (top-level rows, and each row's recursive subrows)
  // This would make multi-filtering a lot faster though. Too far?


  var _React$useMemo = React.useMemo(function () {
    if (manualGlobalFilter || typeof globalFilterValue === 'undefined') {
      return [rows, flatRows];
    }

    var filteredFlatRows = [];
    var filterMethod = getFilterMethod(globalFilter, userFilterTypes || {}, filterTypes);

    if (!filterMethod) {
      console.warn("Could not find a valid 'globalFilter' option.");
      return rows;
    } // Filters top level and nested rows


    var filterRows = function filterRows(filteredRows) {
      return filterMethod(filteredRows, flatColumns.map(function (d) {
        return d.id;
      }), globalFilterValue).map(function (row) {
        filteredFlatRows.push(row);
        return _extends({}, row, {
          subRows: row.subRows && row.subRows.length ? filterRows(row.subRows) : row.subRows
        });
      });
    };

    return [filterRows(rows), filteredFlatRows];
  }, [manualGlobalFilter, globalFilter, userFilterTypes, rows, flatRows, flatColumns, globalFilterValue]),
      globalFilteredRows = _React$useMemo[0],
      globalFilteredFlatRows = _React$useMemo[1];

  var getAutoResetGlobalFilters = useGetLatest(autoResetGlobalFilters);
  useMountedLayoutEffect(function () {
    if (getAutoResetGlobalFilters()) {
      dispatch({
        type: actions.resetGlobalFilter
      });
    }
  }, [dispatch, manualGlobalFilter ? null : data]);
  Object.assign(instance, {
    preGlobalFilteredRows: rows,
    preGlobalFilteredFlatRows: flatRows,
    globalFilteredRows: globalFilteredRows,
    globalFilteredFlatRows: globalFilteredFlatRows,
    rows: globalFilteredRows,
    flatRows: globalFilteredFlatRows,
    setGlobalFilter: setGlobalFilter
  });
}

function sum(values, rows) {
  return values.reduce(function (sum, next) {
    return sum + next;
  }, 0);
}
function average(values, rows) {
  return Math.round(sum(values) / values.length * 100) / 100;
}
function median(values) {
  values = values.length ? values : [0];
  var min = Math.min.apply(Math, values);
  var max = Math.max.apply(Math, values);
  return (min + max) / 2;
}
function uniqueCount(values) {
  return new Set(values).size;
}
function count(values) {
  return values.length;
}

var aggregations = /*#__PURE__*/Object.freeze({
  __proto__: null,
  sum: sum,
  average: average,
  median: median,
  uniqueCount: uniqueCount,
  count: count
});

actions.resetGroupBy = 'resetGroupBy';
actions.toggleGroupBy = 'toggleGroupBy';
var useGroupBy = function useGroupBy(hooks) {
  hooks.getGroupByToggleProps = [defaultGetGroupByToggleProps];
  hooks.stateReducers.push(reducer$4);
  hooks.flatColumnsDeps.push(function (deps, _ref) {
    var instance = _ref.instance;
    return [].concat(deps, [instance.state.groupBy]);
  });
  hooks.flatColumns.push(flatColumns);
  hooks.useInstance.push(useInstance$4);
};
useGroupBy.pluginName = 'useGroupBy';

var defaultGetGroupByToggleProps = function defaultGetGroupByToggleProps(props, _ref2) {
  var header = _ref2.header;
  return [props, {
    onClick: header.canGroupBy ? function (e) {
      e.persist();
      header.toggleGroupBy();
    } : undefined,
    style: {
      cursor: header.canGroupBy ? 'pointer' : undefined
    },
    title: 'Toggle GroupBy'
  }];
}; // Reducer


function reducer$4(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      groupBy: []
    }, state);
  }

  if (action.type === actions.resetGroupBy) {
    return _extends({}, state, {
      groupBy: instance.initialState.groupBy || []
    });
  }

  if (action.type === actions.toggleGroupBy) {
    var columnId = action.columnId,
        toggle = action.toggle;
    var resolvedToggle = typeof toggle !== 'undefined' ? toggle : !state.groupBy.includes(columnId);

    if (resolvedToggle) {
      return _extends({}, state, {
        groupBy: [].concat(state.groupBy, [columnId])
      });
    }

    return _extends({}, state, {
      groupBy: state.groupBy.filter(function (d) {
        return d !== columnId;
      })
    });
  }
}

function flatColumns(flatColumns, _ref3) {
  var groupBy = _ref3.instance.state.groupBy;
  // Sort grouped columns to the start of the column list
  // before the headers are built
  var groupByColumns = groupBy.map(function (g) {
    return flatColumns.find(function (col) {
      return col.id === g;
    });
  }).filter(function (col) {
    return !!col;
  });
  var nonGroupByColumns = flatColumns.filter(function (col) {
    return !groupBy.includes(col.id);
  });
  flatColumns = [].concat(groupByColumns, nonGroupByColumns);
  flatColumns.forEach(function (column) {
    column.isGrouped = groupBy.includes(column.id);
    column.groupedIndex = groupBy.indexOf(column.id);
  });
  return flatColumns;
}

var defaultUserAggregations = {};

function useInstance$4(instance) {
  var data = instance.data,
      rows = instance.rows,
      flatRows = instance.flatRows,
      flatColumns = instance.flatColumns,
      flatHeaders = instance.flatHeaders,
      _instance$groupByFn = instance.groupByFn,
      groupByFn = _instance$groupByFn === void 0 ? defaultGroupByFn : _instance$groupByFn,
      manualGroupBy = instance.manualGroupBy,
      _instance$aggregation = instance.aggregations,
      userAggregations = _instance$aggregation === void 0 ? defaultUserAggregations : _instance$aggregation,
      hooks = instance.hooks,
      plugins = instance.plugins,
      groupBy = instance.state.groupBy,
      dispatch = instance.dispatch,
      _instance$autoResetGr = instance.autoResetGroupBy,
      autoResetGroupBy = _instance$autoResetGr === void 0 ? true : _instance$autoResetGr,
      manaulGroupBy = instance.manaulGroupBy,
      disableGroupBy = instance.disableGroupBy,
      defaultCanGroupBy = instance.defaultCanGroupBy;
  ensurePluginOrder(plugins, [], 'useGroupBy', ['useSortBy', 'useExpanded']);
  var getInstance = useGetLatest(instance);
  flatColumns.forEach(function (column) {
    var accessor = column.accessor,
        defaultColumnGroupBy = column.defaultGroupBy,
        columnDisableGroupBy = column.disableGroupBy;
    column.canGroupBy = accessor ? getFirstDefined(columnDisableGroupBy === true ? false : undefined, disableGroupBy === true ? false : undefined, true) : getFirstDefined(defaultColumnGroupBy, defaultCanGroupBy, false);

    if (column.canGroupBy) {
      column.toggleGroupBy = function () {
        return instance.toggleGroupBy(column.id);
      };
    }

    column.Aggregated = column.Aggregated || column.Cell;
  });

  var toggleGroupBy = function toggleGroupBy(columnId, toggle) {
    dispatch({
      type: actions.toggleGroupBy,
      columnId: columnId,
      toggle: toggle
    });
  };

  var getGroupByTogglePropsHooks = useConsumeHookGetter(getInstance().hooks, 'getGroupByToggleProps');
  flatHeaders.forEach(function (header) {
    header.getGroupByToggleProps = makePropGetter(getGroupByTogglePropsHooks(), {
      instance: getInstance(),
      header: header
    });
  });
  hooks.prepareRow.push(function (row) {
    row.allCells.forEach(function (cell) {
      // Grouped cells are in the groupBy and the pivot cell for the row
      cell.isGrouped = cell.column.isGrouped && cell.column.id === row.groupByID; // Repeated cells are any columns in the groupBy that are not grouped

      cell.isRepeatedValue = !cell.isGrouped && cell.column.isGrouped; // Aggregated cells are not grouped, not repeated, but still have subRows

      cell.isAggregated = !cell.isGrouped && !cell.isRepeatedValue && row.canExpand;
    });
  });

  var _React$useMemo = React.useMemo(function () {
    if (manualGroupBy || !groupBy.length) {
      return [rows, flatRows];
    } // Ensure that the list of filtered columns exist


    var existingGroupBy = groupBy.filter(function (g) {
      return flatColumns.find(function (col) {
        return col.id === g;
      });
    }); // Find the columns that can or are aggregating
    // Uses each column to aggregate rows into a single value

    var aggregateRowsToValues = function aggregateRowsToValues(rows, isAggregated) {
      var values = {};
      flatColumns.forEach(function (column) {
        // Don't aggregate columns that are in the groupBy
        if (existingGroupBy.includes(column.id)) {
          values[column.id] = rows[0] ? rows[0].values[column.id] : null;
          return;
        }

        var columnValues = rows.map(function (d) {
          return d.values[column.id];
        });
        var aggregator = column.aggregate;

        if (Array.isArray(aggregator)) {
          if (aggregator.length !== 2) {
            console.info({
              column: column
            });
            throw new Error("React Table: Complex aggregators must have 2 values, eg. aggregate: ['sum', 'count']. More info above...");
          }

          if (isAggregated) {
            aggregator = aggregator[1];
          } else {
            aggregator = aggregator[0];
          }
        }

        var aggregateFn = typeof aggregator === 'function' ? aggregator : userAggregations[aggregator] || aggregations[aggregator];

        if (aggregateFn) {
          values[column.id] = aggregateFn(columnValues, rows, isAggregated);
        } else if (aggregator) {
          console.info({
            column: column
          });
          throw new Error("React Table: Invalid aggregate option for column listed above");
        } else {
          values[column.id] = null;
        }
      });
      return values;
    };

    var groupedFlatRows = []; // Recursively group the data

    var groupRecursively = function groupRecursively(rows, depth, parentId) {
      if (depth === void 0) {
        depth = 0;
      }

      // This is the last level, just return the rows
      if (depth === existingGroupBy.length) {
        return rows;
      }

      var columnId = existingGroupBy[depth]; // Group the rows together for this level

      var groupedRows = groupByFn(rows, columnId); // Recurse to sub rows before aggregation

      groupedRows = Object.entries(groupedRows).map(function (_ref4, index) {
        var groupByVal = _ref4[0],
            subRows = _ref4[1];
        var id = columnId + ":" + groupByVal;
        id = parentId ? parentId + ">" + id : id;
        subRows = groupRecursively(subRows, depth + 1, id);
        var values = aggregateRowsToValues(subRows, depth < existingGroupBy.length);
        var row = {
          id: id,
          isGrouped: true,
          groupByID: columnId,
          groupByVal: groupByVal,
          values: values,
          subRows: subRows,
          depth: depth,
          index: index
        };
        groupedFlatRows.push.apply(groupedFlatRows, [row].concat(subRows));
        return row;
      });
      return groupedRows;
    };

    var groupedRows = groupRecursively(rows); // Assign the new data

    return [groupedRows, groupedFlatRows];
  }, [manualGroupBy, groupBy, rows, flatRows, flatColumns, userAggregations, groupByFn]),
      groupedRows = _React$useMemo[0],
      groupedFlatRows = _React$useMemo[1];

  var getAutoResetGroupBy = useGetLatest(autoResetGroupBy);
  useMountedLayoutEffect(function () {
    if (getAutoResetGroupBy()) {
      dispatch({
        type: actions.resetGroupBy
      });
    }
  }, [dispatch, manaulGroupBy ? null : data]);
  Object.assign(instance, {
    preGroupedRows: rows,
    preGroupedFlatRow: flatRows,
    groupedRows: groupedRows,
    groupedFlatRows: groupedFlatRows,
    rows: groupedRows,
    flatRows: groupedFlatRows,
    toggleGroupBy: toggleGroupBy
  });
}

var reSplitAlphaNumeric = /([0-9]+)/gm; // Mixed sorting is slow, but very inclusive of many edge cases.
// It handles numbers, mixed alphanumeric combinations, and even
// null, undefined, and Infinity

var alphanumeric = function alphanumeric(rowA, rowB, columnId) {
  var a = getRowValueByColumnID(rowA, columnId);
  var b = getRowValueByColumnID(rowB, columnId); // Force to strings (or "" for unsupported types)

  a = toString(a);
  b = toString(b); // Split on number groups, but keep the delimiter
  // Then remove falsey split values

  a = a.split(reSplitAlphaNumeric).filter(Boolean);
  b = b.split(reSplitAlphaNumeric).filter(Boolean); // While

  while (a.length && b.length) {
    var aa = a.shift();
    var bb = b.shift();
    var an = parseInt(aa, 10);
    var bn = parseInt(bb, 10);
    var combo = [an, bn].sort(); // Both are string

    if (isNaN(combo[0])) {
      if (aa > bb) {
        return 1;
      }

      if (bb > aa) {
        return -1;
      }

      continue;
    } // One is a string, one is a number


    if (isNaN(combo[1])) {
      return isNaN(an) ? -1 : 1;
    } // Both are numbers


    if (an > bn) {
      return 1;
    }

    if (bn > an) {
      return -1;
    }
  }

  return a.length - b.length;
};
function datetime(rowA, rowB, columnId) {
  var a = getRowValueByColumnID(rowA, columnId);
  var b = getRowValueByColumnID(rowB, columnId);
  a = a.getTime();
  b = b.getTime();
  return compareBasic(a, b);
}
function basic(rowA, rowB, columnId) {
  var a = getRowValueByColumnID(rowA, columnId);
  var b = getRowValueByColumnID(rowB, columnId);
  return compareBasic(a, b);
} // Utils

function compareBasic(a, b) {
  return a === b ? 0 : a > b ? 1 : -1;
}

function getRowValueByColumnID(row, columnId) {
  return row.values[columnId];
}

function toString(a) {
  if (typeof a === 'number') {
    if (isNaN(a) || a === Infinity || a === -Infinity) {
      return '';
    }

    return String(a);
  }

  if (typeof a === 'string') {
    return a;
  }

  return '';
}

var sortTypes = /*#__PURE__*/Object.freeze({
  __proto__: null,
  alphanumeric: alphanumeric,
  datetime: datetime,
  basic: basic
});

actions.resetSortBy = 'resetSortBy';
actions.toggleSortBy = 'toggleSortBy';
actions.clearSortBy = 'clearSortBy';
defaultColumn.sortType = 'alphanumeric';
defaultColumn.sortDescFirst = false;
var useSortBy = function useSortBy(hooks) {
  hooks.getSortByToggleProps = [defaultGetSortByToggleProps];
  hooks.stateReducers.push(reducer$5);
  hooks.useInstance.push(useInstance$5);
};
useSortBy.pluginName = 'useSortBy';

var defaultGetSortByToggleProps = function defaultGetSortByToggleProps(props, _ref) {
  var instance = _ref.instance,
      column = _ref.column;
  var _instance$isMultiSort = instance.isMultiSortEvent,
      isMultiSortEvent = _instance$isMultiSort === void 0 ? function (e) {
    return e.shiftKey;
  } : _instance$isMultiSort;
  return [props, {
    onClick: column.canSort ? function (e) {
      e.persist();
      column.toggleSortBy(undefined, !instance.disableMultiSort && isMultiSortEvent(e));
    } : undefined,
    style: {
      cursor: column.canSort ? 'pointer' : undefined
    },
    title: column.canSort ? 'Toggle SortBy' : undefined
  }];
}; // Reducer


function reducer$5(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      sortBy: []
    }, state);
  }

  if (action.type === actions.resetSortBy) {
    return _extends({}, state, {
      sortBy: instance.initialState.sortBy || []
    });
  }

  if (action.type === actions.clearSortBy) {
    var sortBy = state.sortBy;
    var newSortBy = sortBy.filter(function (d) {
      return d.id !== action.columnId;
    });
    return _extends({}, state, {
      sortBy: newSortBy
    });
  }

  if (action.type === actions.toggleSortBy) {
    var columnId = action.columnId,
        desc = action.desc,
        multi = action.multi;
    var flatColumns = instance.flatColumns,
        disableMultiSort = instance.disableMultiSort,
        disableSortRemove = instance.disableSortRemove,
        disableMultiRemove = instance.disableMultiRemove,
        _instance$maxMultiSor = instance.maxMultiSortColCount,
        maxMultiSortColCount = _instance$maxMultiSor === void 0 ? Number.MAX_SAFE_INTEGER : _instance$maxMultiSor;
    var _sortBy = state.sortBy; // Find the column for this columnId

    var column = flatColumns.find(function (d) {
      return d.id === columnId;
    });
    var sortDescFirst = column.sortDescFirst; // Find any existing sortBy for this column

    var existingSortBy = _sortBy.find(function (d) {
      return d.id === columnId;
    });

    var existingIndex = _sortBy.findIndex(function (d) {
      return d.id === columnId;
    });

    var hasDescDefined = typeof desc !== 'undefined' && desc !== null;
    var _newSortBy = []; // What should we do with this sort action?

    var sortAction;

    if (!disableMultiSort && multi) {
      if (existingSortBy) {
        sortAction = 'toggle';
      } else {
        sortAction = 'add';
      }
    } else {
      // Normal mode
      if (existingIndex !== _sortBy.length - 1) {
        sortAction = 'replace';
      } else if (existingSortBy) {
        sortAction = 'toggle';
      } else {
        sortAction = 'replace';
      }
    } // Handle toggle states that will remove the sortBy


    if (sortAction === 'toggle' && // Must be toggling
    !disableSortRemove && // If disableSortRemove, disable in general
    !hasDescDefined && ( // Must not be setting desc
    multi ? !disableMultiRemove : true) && ( // If multi, don't allow if disableMultiRemove
    existingSortBy && // Finally, detect if it should indeed be removed
    existingSortBy.desc && !sortDescFirst || !existingSortBy.desc && sortDescFirst)) {
      sortAction = 'remove';
    }

    if (sortAction === 'replace') {
      _newSortBy = [{
        id: columnId,
        desc: hasDescDefined ? desc : sortDescFirst
      }];
    } else if (sortAction === 'add') {
      _newSortBy = [].concat(_sortBy, [{
        id: columnId,
        desc: hasDescDefined ? desc : sortDescFirst
      }]); // Take latest n columns

      _newSortBy.splice(0, _newSortBy.length - maxMultiSortColCount);
    } else if (sortAction === 'toggle') {
      // This flips (or sets) the
      _newSortBy = _sortBy.map(function (d) {
        if (d.id === columnId) {
          return _extends({}, d, {
            desc: hasDescDefined ? desc : !existingSortBy.desc
          });
        }

        return d;
      });
    } else if (sortAction === 'remove') {
      _newSortBy = _sortBy.filter(function (d) {
        return d.id !== columnId;
      });
    }

    return _extends({}, state, {
      sortBy: _newSortBy
    });
  }
}

function useInstance$5(instance) {
  var data = instance.data,
      rows = instance.rows,
      flatColumns = instance.flatColumns,
      _instance$orderByFn = instance.orderByFn,
      orderByFn = _instance$orderByFn === void 0 ? defaultOrderByFn : _instance$orderByFn,
      userSortTypes = instance.sortTypes,
      manualSortBy = instance.manualSortBy,
      defaultCanSort = instance.defaultCanSort,
      disableSortBy = instance.disableSortBy,
      flatHeaders = instance.flatHeaders,
      sortBy = instance.state.sortBy,
      dispatch = instance.dispatch,
      plugins = instance.plugins,
      _instance$autoResetSo = instance.autoResetSortBy,
      autoResetSortBy = _instance$autoResetSo === void 0 ? true : _instance$autoResetSo;
  ensurePluginOrder(plugins, ['useFilters'], 'useSortBy', []); // Updates sorting based on a columnId, desc flag and multi flag

  var toggleSortBy = function toggleSortBy(columnId, desc, multi) {
    dispatch({
      type: actions.toggleSortBy,
      columnId: columnId,
      desc: desc,
      multi: multi
    });
  }; // use reference to avoid memory leak in #1608


  var getInstance = useGetLatest(instance);
  var getSortByTogglePropsHooks = useConsumeHookGetter(getInstance().hooks, 'getSortByToggleProps'); // Add the getSortByToggleProps method to columns and headers

  flatHeaders.forEach(function (column) {
    var accessor = column.accessor,
        defaultColumnCanSort = column.canSort,
        columnDisableSortBy = column.disableSortBy,
        id = column.id;
    var canSort = accessor ? getFirstDefined(columnDisableSortBy === true ? false : undefined, disableSortBy === true ? false : undefined, true) : getFirstDefined(defaultCanSort, defaultColumnCanSort, false);
    column.canSort = canSort;

    if (column.canSort) {
      column.toggleSortBy = function (desc, multi) {
        return toggleSortBy(column.id, desc, multi);
      };

      column.clearSortBy = function () {
        dispatch({
          type: actions.clearSortBy,
          columnId: column.id
        });
      };
    }

    column.getSortByToggleProps = makePropGetter(getSortByTogglePropsHooks(), {
      instance: getInstance(),
      column: column
    });
    var columnSort = sortBy.find(function (d) {
      return d.id === id;
    });
    column.isSorted = !!columnSort;
    column.sortedIndex = sortBy.findIndex(function (d) {
      return d.id === id;
    });
    column.isSortedDesc = column.isSorted ? columnSort.desc : undefined;
  });
  var sortedRows = React.useMemo(function () {
    if (manualSortBy || !sortBy.length) {
      return rows;
    } // Filter out sortBys that correspond to non existing columns


    var availableSortBy = sortBy.filter(function (sort) {
      return flatColumns.find(function (col) {
        return col.id === sort.id;
      });
    });

    var sortData = function sortData(rows) {
      // Use the orderByFn to compose multiple sortBy's together.
      // This will also perform a stable sorting using the row index
      // if needed.
      var sortedData = orderByFn(rows, availableSortBy.map(function (sort) {
        // Support custom sorting methods for each column
        var column = flatColumns.find(function (d) {
          return d.id === sort.id;
        });

        if (!column) {
          throw new Error("React-Table: Could not find a column with id: " + sort.id + " while sorting");
        }

        var sortType = column.sortType; // Look up sortBy functions in this order:
        // column function
        // column string lookup on user sortType
        // column string lookup on built-in sortType
        // default function
        // default string lookup on user sortType
        // default string lookup on built-in sortType

        var sortMethod = isFunction(sortType) || (userSortTypes || {})[sortType] || sortTypes[sortType];

        if (!sortMethod) {
          throw new Error("React-Table: Could not find a valid sortType of '" + sortType + "' for column '" + sort.id + "'.");
        } // Return the correct sortFn.
        // This function should always return in ascending order


        return function (a, b) {
          return sortMethod(a, b, sort.id);
        };
      }), // Map the directions
      availableSortBy.map(function (sort) {
        // Detect and use the sortInverted option
        var column = flatColumns.find(function (d) {
          return d.id === sort.id;
        });

        if (column && column.sortInverted) {
          return sort.desc;
        }

        return !sort.desc;
      })); // If there are sub-rows, sort them

      sortedData.forEach(function (row) {
        if (!row.subRows || row.subRows.length <= 1) {
          return;
        }

        row.subRows = sortData(row.subRows);
      });
      return sortedData;
    };

    return sortData(rows);
  }, [manualSortBy, sortBy, rows, flatColumns, orderByFn, userSortTypes]);
  var getAutoResetSortBy = useGetLatest(autoResetSortBy);
  useMountedLayoutEffect(function () {
    if (getAutoResetSortBy()) {
      dispatch({
        type: actions.resetSortBy
      });
    }
  }, [manualSortBy ? null : data]);
  Object.assign(instance, {
    preSortedRows: rows,
    sortedRows: sortedRows,
    rows: sortedRows,
    toggleSortBy: toggleSortBy
  });
}

var pluginName = 'usePagination'; // Actions

actions.resetPage = 'resetPage';
actions.gotoPage = 'gotoPage';
actions.setPageSize = 'setPageSize';
var usePagination = function usePagination(hooks) {
  hooks.stateReducers.push(reducer$6);
  hooks.useInstance.push(useInstance$6);
};
usePagination.pluginName = pluginName;

function reducer$6(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      pageSize: 10,
      pageIndex: 0
    }, state);
  }

  if (action.type === actions.resetPage) {
    return _extends({}, state, {
      pageIndex: instance.initialState.pageIndex || 0
    });
  }

  if (action.type === actions.gotoPage) {
    var pageCount = instance.pageCount;
    var newPageIndex = functionalUpdate(action.pageIndex, state.pageIndex);

    if (newPageIndex < 0 || newPageIndex > pageCount - 1) {
      return state;
    }

    return _extends({}, state, {
      pageIndex: newPageIndex
    });
  }

  if (action.type === actions.setPageSize) {
    var pageSize = action.pageSize;
    var topRowIndex = state.pageSize * state.pageIndex;
    var pageIndex = Math.floor(topRowIndex / pageSize);
    return _extends({}, state, {
      pageIndex: pageIndex,
      pageSize: pageSize
    });
  }
}

function useInstance$6(instance) {
  var rows = instance.rows,
      _instance$autoResetPa = instance.autoResetPage,
      autoResetPage = _instance$autoResetPa === void 0 ? true : _instance$autoResetPa,
      _instance$manualExpan = instance.manualExpandedKey,
      manualExpandedKey = _instance$manualExpan === void 0 ? 'expanded' : _instance$manualExpan,
      plugins = instance.plugins,
      userPageCount = instance.pageCount,
      _instance$paginateExp = instance.paginateExpandedRows,
      paginateExpandedRows = _instance$paginateExp === void 0 ? true : _instance$paginateExp,
      _instance$expandSubRo = instance.expandSubRows,
      expandSubRows = _instance$expandSubRo === void 0 ? true : _instance$expandSubRo,
      _instance$state = instance.state,
      pageSize = _instance$state.pageSize,
      pageIndex = _instance$state.pageIndex,
      expanded = _instance$state.expanded,
      filters = _instance$state.filters,
      groupBy = _instance$state.groupBy,
      sortBy = _instance$state.sortBy,
      dispatch = instance.dispatch,
      data = instance.data,
      manualPagination = instance.manualPagination,
      manualFilters = instance.manualFilters,
      manualGroupBy = instance.manualGroupBy,
      manualSortBy = instance.manualSortBy;
  ensurePluginOrder(plugins, ['useFilters', 'useGroupBy', 'useSortBy', 'useExpanded'], 'usePagination', []);
  var getAutoResetPage = useGetLatest(autoResetPage);
  useMountedLayoutEffect(function () {
    if (getAutoResetPage()) {
      dispatch({
        type: actions.resetPage
      });
    }
  }, [dispatch, manualPagination ? null : data, manualPagination || manualFilters ? null : filters, manualPagination || manualGroupBy ? null : groupBy, manualPagination || manualSortBy ? null : sortBy]);
  var pageCount = manualPagination ? userPageCount : Math.ceil(rows.length / pageSize);
  var pageOptions = React.useMemo(function () {
    return pageCount > 0 ? [].concat(new Array(pageCount)).map(function (d, i) {
      return i;
    }) : [];
  }, [pageCount]);
  var page = React.useMemo(function () {
    var page;

    if (manualPagination) {
      page = rows;
    } else {
      var pageStart = pageSize * pageIndex;
      var pageEnd = pageStart + pageSize;
      page = rows.slice(pageStart, pageEnd);
    }

    if (paginateExpandedRows) {
      return page;
    }

    return expandRows(page, {
      manualExpandedKey: manualExpandedKey,
      expanded: expanded,
      expandSubRows: expandSubRows
    });
  }, [expandSubRows, expanded, manualExpandedKey, manualPagination, pageIndex, pageSize, paginateExpandedRows, rows]);
  var canPreviousPage = pageIndex > 0;
  var canNextPage = pageCount === -1 || pageIndex < pageCount - 1;
  var gotoPage = React.useCallback(function (pageIndex) {
    dispatch({
      type: actions.gotoPage,
      pageIndex: pageIndex
    });
  }, [dispatch]);
  var previousPage = React.useCallback(function () {
    return gotoPage(function (old) {
      return old - 1;
    });
  }, [gotoPage]);
  var nextPage = React.useCallback(function () {
    return gotoPage(function (old) {
      return old + 1;
    });
  }, [gotoPage]);
  var setPageSize = React.useCallback(function (pageSize) {
    dispatch({
      type: actions.setPageSize,
      pageSize: pageSize
    });
  }, [dispatch]);
  Object.assign(instance, {
    pageOptions: pageOptions,
    pageCount: pageCount,
    page: page,
    canPreviousPage: canPreviousPage,
    canNextPage: canNextPage,
    gotoPage: gotoPage,
    previousPage: previousPage,
    nextPage: nextPage,
    setPageSize: setPageSize
  });
}

var pluginName$1 = 'useRowSelect'; // Actions

actions.resetSelectedRows = 'resetSelectedRows';
actions.toggleAllRowsSelected = 'toggleAllRowsSelected';
actions.toggleRowSelected = 'toggleRowSelected';
var useRowSelect = function useRowSelect(hooks) {
  hooks.getToggleRowSelectedProps = [defaultGetToggleRowSelectedProps];
  hooks.getToggleAllRowsSelectedProps = [defaultGetToggleAllRowsSelectedProps];
  hooks.stateReducers.push(reducer$7);
  hooks.useRows.push(useRows);
  hooks.useInstance.push(useInstance$7);
};
useRowSelect.pluginName = pluginName$1;

var defaultGetToggleRowSelectedProps = function defaultGetToggleRowSelectedProps(props, _ref) {
  var instance = _ref.instance,
      row = _ref.row;
  var _instance$manualRowSe = instance.manualRowSelectedKey,
      manualRowSelectedKey = _instance$manualRowSe === void 0 ? 'isSelected' : _instance$manualRowSe;
  var checked = false;

  if (row.original && row.original[manualRowSelectedKey]) {
    checked = true;
  } else {
    checked = row.isSelected;
  }

  return [props, {
    onChange: function onChange(e) {
      row.toggleRowSelected(e.target.checked);
    },
    style: {
      cursor: 'pointer'
    },
    checked: checked,
    title: 'Toggle Row Selected',
    indeterminate: row.isSomeSelected
  }];
};

var defaultGetToggleAllRowsSelectedProps = function defaultGetToggleAllRowsSelectedProps(props, _ref2) {
  var instance = _ref2.instance;
  return [props, {
    onChange: function onChange(e) {
      instance.toggleAllRowsSelected(e.target.checked);
    },
    style: {
      cursor: 'pointer'
    },
    checked: instance.isAllRowsSelected,
    title: 'Toggle All Rows Selected',
    indeterminate: Boolean(!instance.isAllRowsSelected && Object.keys(instance.state.selectedRowIds).length)
  }];
};

function reducer$7(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      selectedRowIds: {}
    }, state);
  }

  if (action.type === actions.resetSelectedRows) {
    return _extends({}, state, {
      selectedRowIds: instance.initialState.selectedRowIds || {}
    });
  }

  if (action.type === actions.toggleAllRowsSelected) {
    var selected = action.selected;
    var isAllRowsSelected = instance.isAllRowsSelected,
        flatRowsById = instance.flatRowsById;
    var selectAll = typeof selected !== 'undefined' ? selected : !isAllRowsSelected;

    if (selectAll) {
      var selectedRowIds = {};
      Object.keys(flatRowsById).forEach(function (rowId) {
        selectedRowIds[rowId] = true;
      });
      return _extends({}, state, {
        selectedRowIds: selectedRowIds
      });
    }

    return _extends({}, state, {
      selectedRowIds: {}
    });
  }

  if (action.type === actions.toggleRowSelected) {
    var id = action.id,
        _selected = action.selected;
    var flatGroupedRowsById = instance.flatGroupedRowsById; // Join the ids of deep rows
    // to make a key, then manage all of the keys
    // in a flat object

    var row = flatGroupedRowsById[id];
    var isSelected = row.isSelected;
    var shouldExist = typeof _selected !== 'undefined' ? _selected : !isSelected;

    if (isSelected === shouldExist) {
      return state;
    }

    var newSelectedRowIds = _extends({}, state.selectedRowIds);

    var handleRowById = function handleRowById(id) {
      var row = flatGroupedRowsById[id];

      if (!row.isGrouped) {
        if (!isSelected && shouldExist) {
          newSelectedRowIds[id] = true;
        } else if (isSelected && !shouldExist) {
          delete newSelectedRowIds[id];
        }
      }

      if (row.subRows) {
        return row.subRows.forEach(function (row) {
          return handleRowById(row.id);
        });
      }
    };

    handleRowById(id);
    return _extends({}, state, {
      selectedRowIds: newSelectedRowIds
    });
  }
}

function useRows(rows, _ref3) {
  var instance = _ref3.instance;
  var selectedRowIds = instance.state.selectedRowIds;
  instance.selectedFlatRows = React.useMemo(function () {
    var selectedFlatRows = [];
    rows.forEach(function (row) {
      var isSelected = getRowIsSelected(row, selectedRowIds);
      row.isSelected = !!isSelected;
      row.isSomeSelected = isSelected === null;

      if (isSelected) {
        selectedFlatRows.push(row);
      }
    });
    return selectedFlatRows;
  }, [rows, selectedRowIds]);
  return rows;
}

function useInstance$7(instance) {
  var data = instance.data,
      hooks = instance.hooks,
      plugins = instance.plugins,
      flatRows = instance.flatRows,
      _instance$autoResetSe = instance.autoResetSelectedRows,
      autoResetSelectedRows = _instance$autoResetSe === void 0 ? true : _instance$autoResetSe,
      selectedRowIds = instance.state.selectedRowIds,
      dispatch = instance.dispatch;
  ensurePluginOrder(plugins, ['useFilters', 'useGroupBy', 'useSortBy'], 'useRowSelect', []);

  var _React$useMemo = React.useMemo(function () {
    var all = {};
    var grouped = {};
    flatRows.forEach(function (row) {
      if (!row.isGrouped) {
        all[row.id] = row;
      }

      grouped[row.id] = row;
    });
    return [all, grouped];
  }, [flatRows]),
      flatRowsById = _React$useMemo[0],
      flatGroupedRowsById = _React$useMemo[1];

  var isAllRowsSelected = Boolean(Object.keys(flatRowsById).length && Object.keys(selectedRowIds).length);

  if (isAllRowsSelected) {
    if (Object.keys(flatRowsById).some(function (id) {
      return !selectedRowIds[id];
    })) {
      isAllRowsSelected = false;
    }
  }

  var getAutoResetSelectedRows = useGetLatest(autoResetSelectedRows);
  useMountedLayoutEffect(function () {
    if (getAutoResetSelectedRows()) {
      dispatch({
        type: actions.resetSelectedRows
      });
    }
  }, [dispatch, data]);

  var toggleAllRowsSelected = function toggleAllRowsSelected(selected) {
    return dispatch({
      type: actions.toggleAllRowsSelected,
      selected: selected
    });
  };

  var toggleRowSelected = function toggleRowSelected(id, selected) {
    return dispatch({
      type: actions.toggleRowSelected,
      id: id,
      selected: selected
    });
  };

  var getInstance = useGetLatest(instance);
  var getToggleAllRowsSelectedPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getToggleAllRowsSelectedProps');
  var getToggleAllRowsSelectedProps = makePropGetter(getToggleAllRowsSelectedPropsHooks(), {
    instance: getInstance()
  });
  var getToggleRowSelectedPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getToggleRowSelectedProps');
  hooks.prepareRow.push(function (row) {
    row.toggleRowSelected = function (set) {
      return toggleRowSelected(row.id, set);
    };

    row.getToggleRowSelectedProps = makePropGetter(getToggleRowSelectedPropsHooks(), {
      instance: getInstance(),
      row: row
    });
  });
  Object.assign(instance, {
    flatRowsById: flatRowsById,
    flatGroupedRowsById: flatGroupedRowsById,
    toggleRowSelected: toggleRowSelected,
    toggleAllRowsSelected: toggleAllRowsSelected,
    getToggleAllRowsSelectedProps: getToggleAllRowsSelectedProps,
    isAllRowsSelected: isAllRowsSelected
  });
}

function getRowIsSelected(row, selectedRowIds) {
  if (selectedRowIds[row.id]) {
    return true;
  }

  if (row.subRows && row.subRows.length) {
    var allChildrenSelected = true;
    var someSelected = false;
    row.subRows.forEach(function (subRow) {
      // Bail out early if we know both of these
      if (someSelected && !allChildrenSelected) {
        return;
      }

      if (getRowIsSelected(subRow, selectedRowIds)) {
        someSelected = true;
      } else {
        allChildrenSelected = false;
      }
    });
    return allChildrenSelected ? true : someSelected ? null : false;
  }

  return false;
}

actions.setRowState = 'setRowState';
actions.resetRowState = 'resetRowState';
var useRowState = function useRowState(hooks) {
  hooks.stateReducers.push(reducer$8);
  hooks.useInstance.push(useInstance$8);
};
useRowState.pluginName = 'useRowState';

function reducer$8(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      rowState: {}
    }, state);
  }

  if (action.type === actions.resetRowState) {
    return _extends({}, state, {
      rowState: instance.initialState.rowState || {}
    });
  }

  if (action.type === actions.setRowState) {
    var _extends2;

    var id = action.id,
        value = action.value;
    return _extends({}, state, {
      rowState: _extends({}, state.rowState, (_extends2 = {}, _extends2[id] = functionalUpdate(value, state.rowState[id] || {}), _extends2))
    });
  }
}

function useInstance$8(instance) {
  var hooks = instance.hooks,
      initialRowStateAccessor = instance.initialRowStateAccessor,
      _instance$autoResetRo = instance.autoResetRowState,
      autoResetRowState = _instance$autoResetRo === void 0 ? true : _instance$autoResetRo,
      rowState = instance.state.rowState,
      data = instance.data,
      dispatch = instance.dispatch;
  var setRowState = React.useCallback(function (id, value, columnId) {
    return dispatch({
      type: actions.setRowState,
      id: id,
      value: value,
      columnId: columnId
    });
  }, [dispatch]);
  var setCellState = React.useCallback(function (rowPath, columnId, value) {
    return setRowState(rowPath, function (old) {
      var _extends3;

      return _extends({}, old, {
        cellState: _extends({}, old.cellState, (_extends3 = {}, _extends3[columnId] = functionalUpdate(value, (old.cellState || {})[columnId] || {}), _extends3))
      });
    }, columnId);
  }, [setRowState]);
  hooks.prepareRow.push(function (row) {
    if (row.original) {
      row.state = (typeof rowState[row.id] !== 'undefined' ? rowState[row.id] : initialRowStateAccessor && initialRowStateAccessor(row)) || {};

      row.setState = function (updater) {
        return setRowState(row.id, updater);
      };

      row.cells.forEach(function (cell) {
        cell.state = row.state.cellState || {};

        cell.setState = function (updater) {
          return setCellState(row.id, cell.column.id, updater);
        };
      });
    }
  });
  var getAutoResetRowState = useGetLatest(autoResetRowState);
  useMountedLayoutEffect(function () {
    if (getAutoResetRowState()) {
      dispatch({
        type: actions.resetRowState
      });
    }
  }, [data]);
  Object.assign(instance, {
    setRowState: setRowState,
    setCellState: setCellState
  });
}

actions.resetColumnOrder = 'resetColumnOrder';
actions.setColumnOrder = 'setColumnOrder';
var useColumnOrder = function useColumnOrder(hooks) {
  hooks.stateReducers.push(reducer$9);
  hooks.flatColumnsDeps.push(function (deps, _ref) {
    var instance = _ref.instance;
    return [].concat(deps, [instance.state.columnOrder]);
  });
  hooks.flatColumns.push(flatColumns$1);
  hooks.useInstance.push(useInstance$9);
};
useColumnOrder.pluginName = 'useColumnOrder';

function reducer$9(state, action, previousState, instance) {
  if (action.type === actions.init) {
    return _extends({
      columnOrder: []
    }, state);
  }

  if (action.type === actions.resetColumnOrder) {
    return _extends({}, state, {
      columnOrder: instance.initialState.columnOrder || []
    });
  }

  if (action.type === actions.setColumnOrder) {
    return _extends({}, state, {
      columnOrder: functionalUpdate(action.columnOrder, state.columnOrder)
    });
  }
}

function flatColumns$1(columns, _ref2) {
  var columnOrder = _ref2.instance.state.columnOrder;

  // If there is no order, return the normal columns
  if (!columnOrder || !columnOrder.length) {
    return columns;
  }

  var columnOrderCopy = [].concat(columnOrder); // If there is an order, make a copy of the columns

  var columnsCopy = [].concat(columns); // And make a new ordered array of the columns

  var columnsInOrder = []; // Loop over the columns and place them in order into the new array

  var _loop = function _loop() {
    var targetColumnId = columnOrderCopy.shift();
    var foundIndex = columnsCopy.findIndex(function (d) {
      return d.id === targetColumnId;
    });

    if (foundIndex > -1) {
      columnsInOrder.push(columnsCopy.splice(foundIndex, 1)[0]);
    }
  };

  while (columnsCopy.length && columnOrderCopy.length) {
    _loop();
  } // If there are any columns left, add them to the end


  return [].concat(columnsInOrder, columnsCopy);
}

function useInstance$9(instance) {
  var dispatch = instance.dispatch;
  instance.setColumnOrder = React.useCallback(function (columnOrder) {
    return dispatch({
      type: actions.setColumnOrder,
      columnOrder: columnOrder
    });
  }, [dispatch]);
}

defaultColumn.canResize = true; // Actions

actions.columnStartResizing = 'columnStartResizing';
actions.columnResizing = 'columnResizing';
actions.columnDoneResizing = 'columnDoneResizing';
var useResizeColumns = function useResizeColumns(hooks) {
  hooks.getResizerProps = [defaultGetResizerProps];
  hooks.getHeaderProps.push({
    style: {
      position: 'relative'
    }
  });
  hooks.stateReducers.push(reducer$a);
  hooks.useInstanceBeforeDimensions.push(useInstanceBeforeDimensions$1);
};

var defaultGetResizerProps = function defaultGetResizerProps(props, _ref) {
  var instance = _ref.instance,
      header = _ref.header;
  var dispatch = instance.dispatch;

  var onResizeStart = function onResizeStart(e, header) {
    var isTouchEvent = false;

    if (e.type === 'touchstart') {
      // lets not respond to multiple touches (e.g. 2 or 3 fingers)
      if (e.touches && e.touches.length > 1) {
        return;
      }

      isTouchEvent = true;
    }

    var headersToResize = getLeafHeaders(header);
    var headerIdWidths = headersToResize.map(function (d) {
      return [d.id, d.totalWidth];
    });
    var clientX = isTouchEvent ? Math.round(e.touches[0].clientX) : e.clientX;

    var dispatchMove = function dispatchMove(clientXPos) {
      dispatch({
        type: actions.columnResizing,
        clientX: clientXPos
      });
    };

    var dispatchEnd = function dispatchEnd() {
      return dispatch({
        type: actions.columnDoneResizing
      });
    };

    var handlersAndEvents = {
      mouse: {
        moveEvent: 'mousemove',
        moveHandler: function moveHandler(e) {
          return dispatchMove(e.clientX);
        },
        upEvent: 'mouseup',
        upHandler: function upHandler(e) {
          document.removeEventListener('mousemove', handlersAndEvents.mouse.moveHandler);
          document.removeEventListener('mouseup', handlersAndEvents.mouse.upHandler);
          dispatchEnd();
        }
      },
      touch: {
        moveEvent: 'touchmove',
        moveHandler: function moveHandler(e) {
          if (e.cancelable) {
            e.preventDefault();
            e.stopPropagation();
          }

          dispatchMove(e.touches[0].clientX);
          return false;
        },
        upEvent: 'touchend',
        upHandler: function upHandler(e) {
          document.removeEventListener(handlersAndEvents.touch.moveEvent, handlersAndEvents.touch.moveHandler);
          document.removeEventListener(handlersAndEvents.touch.upEvent, handlersAndEvents.touch.moveHandler);
          dispatchEnd();
        }
      }
    };
    var events = isTouchEvent ? handlersAndEvents.touch : handlersAndEvents.mouse;
    document.addEventListener(events.moveEvent, events.moveHandler, {
      passive: false
    });
    document.addEventListener(events.upEvent, events.upHandler, {
      passive: false
    });
    dispatch({
      type: actions.columnStartResizing,
      columnId: header.id,
      columnWidth: header.totalWidth,
      headerIdWidths: headerIdWidths,
      clientX: clientX
    });
  };

  return [props, {
    onMouseDown: function onMouseDown(e) {
      return e.persist() || onResizeStart(e, header);
    },
    onTouchStart: function onTouchStart(e) {
      return e.persist() || onResizeStart(e, header);
    },
    style: {
      cursor: 'ew-resize'
    },
    draggable: false
  }];
};

useResizeColumns.pluginName = 'useResizeColumns';

function reducer$a(state, action) {
  if (action.type === actions.init) {
    return _extends({
      columnResizing: {
        columnWidths: {}
      }
    }, state);
  }

  if (action.type === actions.columnStartResizing) {
    var clientX = action.clientX,
        columnId = action.columnId,
        columnWidth = action.columnWidth,
        headerIdWidths = action.headerIdWidths;
    return _extends({}, state, {
      columnResizing: _extends({}, state.columnResizing, {
        startX: clientX,
        headerIdWidths: headerIdWidths,
        columnWidth: columnWidth,
        isResizingColumn: columnId
      })
    });
  }

  if (action.type === actions.columnResizing) {
    var _clientX = action.clientX;
    var _state$columnResizing = state.columnResizing,
        startX = _state$columnResizing.startX,
        _columnWidth = _state$columnResizing.columnWidth,
        _headerIdWidths = _state$columnResizing.headerIdWidths;
    var deltaX = _clientX - startX;
    var percentageDeltaX = deltaX / _columnWidth;
    var newColumnWidths = {};

    _headerIdWidths.forEach(function (_ref2) {
      var headerId = _ref2[0],
          headerWidth = _ref2[1];
      newColumnWidths[headerId] = Math.max(headerWidth + headerWidth * percentageDeltaX, 0);
    });

    return _extends({}, state, {
      columnResizing: _extends({}, state.columnResizing, {
        columnWidths: _extends({}, state.columnResizing.columnWidths, {}, newColumnWidths)
      })
    });
  }

  if (action.type === actions.columnDoneResizing) {
    return _extends({}, state, {
      columnResizing: _extends({}, state.columnResizing, {
        startX: null,
        isResizingColumn: null
      })
    });
  }
}

var useInstanceBeforeDimensions$1 = function useInstanceBeforeDimensions(instance) {
  var flatHeaders = instance.flatHeaders,
      disableResizing = instance.disableResizing,
      columnResizing = instance.state.columnResizing;
  var getInstance = useGetLatest(instance);
  var getResizerPropsHooks = useConsumeHookGetter(getInstance().hooks, 'getResizerProps');
  flatHeaders.forEach(function (header) {
    var canResize = getFirstDefined(header.disableResizing === true ? false : undefined, disableResizing === true ? false : undefined, true);
    header.canResize = canResize;
    header.width = columnResizing.columnWidths[header.id] || header.width;
    header.isResizing = columnResizing.isResizingColumn === header.id;

    if (canResize) {
      header.getResizerProps = makePropGetter(getResizerPropsHooks(), {
        instance: getInstance(),
        header: header
      });
    }
  });
};

function getLeafHeaders(header) {
  var leafHeaders = [];

  var recurseHeader = function recurseHeader(header) {
    if (header.columns && header.columns.length) {
      header.columns.map(recurseHeader);
    }

    leafHeaders.push(header);
  };

  recurseHeader(header);
  return leafHeaders;
}

var cellStyles = {
  position: 'absolute',
  top: 0
};
var useAbsoluteLayout = function useAbsoluteLayout(hooks) {
  hooks.getTableBodyProps.push(getRowStyles);
  hooks.getRowProps.push(getRowStyles);
  hooks.getHeaderGroupProps.push(getRowStyles);
  hooks.useInstance.push(useInstance$a);
  hooks.getHeaderProps.push(function (props, _ref) {
    var column = _ref.column;
    return [props, {
      style: _extends({}, cellStyles, {
        left: column.totalLeft + "px",
        width: column.totalWidth + "px"
      })
    }];
  });
  hooks.getCellProps.push(function (props, _ref2) {
    var cell = _ref2.cell;
    return [props, {
      style: _extends({}, cellStyles, {
        left: cell.column.totalLeft + "px",
        width: cell.column.totalWidth + "px"
      })
    }];
  });
};
useAbsoluteLayout.pluginName = 'useAbsoluteLayout';

var getRowStyles = function getRowStyles(props, _ref3) {
  var instance = _ref3.instance;
  return [props, {
    style: {
      position: 'relative',
      width: instance.totalColumnsWidth + "px"
    }
  }];
};

function useInstance$a(_ref4) {
  var plugins = _ref4.plugins;
  ensurePluginOrder(plugins, [], useAbsoluteLayout.pluginName, ['useResizeColumns']);
}

var cellStyles$1 = {
  display: 'inline-block',
  boxSizing: 'border-box'
};

var getRowStyles$1 = function getRowStyles(props, _ref) {
  var instance = _ref.instance;
  return [props, {
    style: {
      display: 'flex',
      width: instance.totalColumnsWidth + "px"
    }
  }];
};

var useBlockLayout = function useBlockLayout(hooks) {
  hooks.getRowProps.push(getRowStyles$1);
  hooks.getHeaderGroupProps.push(getRowStyles$1);
  hooks.getHeaderProps.push(function (props, _ref2) {
    var column = _ref2.column;
    return [props, {
      style: _extends({}, cellStyles$1, {
        width: column.totalWidth + "px"
      })
    }];
  });
  hooks.getCellProps.push(function (props, _ref3) {
    var cell = _ref3.cell;
    return [props, {
      style: _extends({}, cellStyles$1, {
        width: cell.column.totalWidth + "px"
      })
    }];
  });
};
useBlockLayout.pluginName = 'useBlockLayout';

function useFlexLayout(hooks) {
  hooks.getTableBodyProps.push(getTableBodyProps);
  hooks.getRowProps.push(getRowStyles$2);
  hooks.getHeaderGroupProps.push(getRowStyles$2);
  hooks.getHeaderProps.push(getHeaderProps);
  hooks.getCellProps.push(getCellProps);
}
useFlexLayout.pluginName = 'useFlexLayout';

var getTableBodyProps = function getTableBodyProps(props, _ref) {
  var instance = _ref.instance;
  return [props, {
    style: {
      minWidth: instance.totalColumnsWidth + "px"
    }
  }];
};

var getRowStyles$2 = function getRowStyles(props, _ref2) {
  var instance = _ref2.instance;
  return [props, {
    style: {
      display: 'flex',
      flex: '1 0 auto',
      minWidth: instance.totalColumnsMinWidth + "px"
    }
  }];
};

var getHeaderProps = function getHeaderProps(props, _ref3) {
  var column = _ref3.column;
  return [props, {
    style: {
      boxSizing: 'border-box',
      flex: column.totalWidth + " 0 auto",
      minWidth: column.totalMinWidth + "px",
      width: column.totalWidth + "px"
    }
  }];
};

var getCellProps = function getCellProps(props, _ref4) {
  var cell = _ref4.cell;
  return [props, {
    style: {
      boxSizing: 'border-box',
      flex: cell.column.totalWidth + " 0 auto",
      minWidth: cell.column.totalMinWidth + "px",
      width: cell.column.totalWidth + "px"
    }
  }];
};

export { actions, defaultColumn, defaultGroupByFn, defaultOrderByFn, ensurePluginOrder, flexRender, functionalUpdate, loopHooks, makePropGetter, makeRenderer, reduceHooks, safeUseLayoutEffect, useAbsoluteLayout, useAsyncDebounce, useBlockLayout, useColumnOrder, useConsumeHookGetter, useExpanded, useFilters, useFlexLayout, useGetLatest, useGlobalFilter, useGroupBy, useMountedLayoutEffect, usePagination, useResizeColumns, useRowSelect, useRowState, useSortBy, useTable };
//# sourceMappingURL=index.es.js.map

import * as React from '/static/js/depend/react/react.js'

const VALIDATION_MODE = {
    onBlur: 'onBlur',
    onChange: 'onChange',
    onSubmit: 'onSubmit',
};
const RADIO_INPUT = 'radio';
const UNDEFINED = 'undefined';
const EVENTS = {
    BLUR: 'blur',
    CHANGE: 'change',
    INPUT: 'input',
};
const INPUT_VALIDATION_RULES = {
    max: 'max',
    min: 'min',
    maxLength: 'maxLength',
    minLength: 'minLength',
    pattern: 'pattern',
    required: 'required',
    validate: 'validate',
};

function attachEventListeners({ field, validateAndStateUpdate, isRadioOrCheckbox, }) {
    const { ref } = field;
    if (!ref.addEventListener) {
        return;
    }
    ref.addEventListener(isRadioOrCheckbox ? EVENTS.CHANGE : EVENTS.INPUT, validateAndStateUpdate);
    ref.addEventListener(EVENTS.BLUR, validateAndStateUpdate);
}

var isUndefined = (val) => val === undefined;

var isNullOrUndefined = (value) => value === null || isUndefined(value);

var isArray = (value) => Array.isArray(value);

const isObjectType = (value) => typeof value === 'object';
var isObject = (value) => !isNullOrUndefined(value) && !isArray(value) && isObjectType(value);

const reIsDeepProp = /\.|\[(?:[^[\]]*|(["'])(?:(?!\1)[^\\]|\\.)*?\1)\]/;
const reIsPlainProp = /^\w*$/;
const rePropName = /[^.[\]]+|\[(?:(-?\d+(?:\.\d+)?)|(["'])((?:(?!\2)[^\\]|\\.)*?)\2)\]|(?=(?:\.|\[\])(?:\.|\[\]|$))/g;
const reEscapeChar = /\\(\\)?/g;
const reIsUint = /^(?:0|[1-9]\d*)$/;
function isIndex(value) {
    return reIsUint.test(value) && value > -1;
}
function isKey(value) {
    if (isArray(value)) {
        return false;
    }
    return reIsPlainProp.test(value) || !reIsDeepProp.test(value);
}
const stringToPath = (string) => {
    const result = [];
    string.replace(rePropName, (match, number, quote, string) => {
        result.push(quote ? string.replace(reEscapeChar, '$1') : number || match);
    });
    return result;
};
function set(object, path, value) {
    let index = -1;
    const tempPath = isKey(path) ? [path] : stringToPath(path);
    const length = tempPath.length;
    const lastIndex = length - 1;
    while (++index < length) {
        const key = tempPath[index];
        let newValue = value;
        if (index !== lastIndex) {
            const objValue = object[key];
            newValue =
                isObject(objValue) || isArray(objValue)
                    ? objValue
                    : isIndex(tempPath[index + 1])
                        ? []
                        : {};
        }
        object[key] = newValue;
        object = object[key];
    }
    return object;
}

var combineFieldValues = (data) => Object.entries(data).reduce((previous, [key, value]) => {
    if (!!key.match(/\[.+\]/gi) || key.indexOf('.') > 0) {
        set(previous, key, value);
        return previous;
    }
    return Object.assign(Object.assign({}, previous), { [key]: value });
}, {});

var removeAllEventListeners = (ref, validateWithStateUpdate) => {
    if (!ref.removeEventListener) {
        return;
    }
    ref.removeEventListener(EVENTS.INPUT, validateWithStateUpdate);
    ref.removeEventListener(EVENTS.CHANGE, validateWithStateUpdate);
    ref.removeEventListener(EVENTS.BLUR, validateWithStateUpdate);
};

var isRadioInput = (type) => type === RADIO_INPUT;

var isCheckBoxInput = (type) => type === 'checkbox';

function isDetached(element) {
    if (!element) {
        return true;
    }
    if (!(element instanceof HTMLElement) ||
        element.nodeType === Node.DOCUMENT_NODE) {
        return false;
    }
    return isDetached(element.parentNode);
}

function findRemovedFieldAndRemoveListener(fields, validateWithStateUpdate = () => { }, field, forceDelete) {
    if (!field) {
        return;
    }
    const { ref, mutationWatcher } = field;
    if (!ref.type || !fields[ref.name]) {
        return;
    }
    const { name, type } = ref;
    const options = fields[name];
    if (isRadioInput(type) || isCheckBoxInput(type)) {
        if (isArray(options) && options.length) {
            options.forEach(({ ref }, index) => {
                const option = options[index];
                if ((option && isDetached(ref)) || forceDelete) {
                    const mutationWatcher = option.mutationWatcher;
                    removeAllEventListeners(option, validateWithStateUpdate);
                    if (mutationWatcher) {
                        mutationWatcher.disconnect();
                    }
                    options.splice(index, 1);
                }
            });
        }
        else {
            delete fields[name];
        }
    }
    else if (isDetached(ref) || forceDelete) {
        removeAllEventListeners(ref, validateWithStateUpdate);
        if (mutationWatcher) {
            mutationWatcher.disconnect();
        }
        delete fields[name];
    }
}

const defaultReturn = {
    isValid: false,
    value: '',
};
var getRadioValue = (options) => isArray(options)
    ? options.reduce((previous, { ref: { checked, value } }) => checked
        ? {
            isValid: true,
            value,
        }
        : previous, defaultReturn)
    : defaultReturn;

var getMultipleSelectValue = (options) => [...options]
    .filter(({ selected }) => selected)
    .map(({ value }) => value);

var isMultipleSelect = (type) => type === 'select-multiple';

var isEmptyString = (value) => value === '';

const defaultResult = {
    value: false,
    isValid: false,
};
const validResult = { value: true, isValid: true };
var getCheckboxValue = (options) => {
    if (isArray(options)) {
        if (options.length > 1) {
            const values = options
                .filter(({ ref: { checked } }) => checked)
                .map(({ ref: { value } }) => value);
            return { value: values, isValid: !!values.length };
        }
        const { checked, value, attributes: { value: valueAttribute }, } = options[0].ref;
        return checked
            ? valueAttribute
                ? isUndefined(value) || isEmptyString(value)
                    ? validResult
                    : { value: value, isValid: true }
                : validResult
            : defaultResult;
    }
    return defaultResult;
};

function getFieldValue(fields, ref) {
    const { type, name, options, value, files } = ref;
    const field = fields[name];
    if (type === 'file') {
        return files;
    }
    if (isRadioInput(type)) {
        return field ? getRadioValue(field.options).value : '';
    }
    if (isMultipleSelect(type)) {
        return getMultipleSelectValue(options);
    }
    if (isCheckBoxInput(type)) {
        return field ? getCheckboxValue(field.options).value : false;
    }
    return value;
}

var getFieldsValues = (fields) => Object.values(fields).reduce((previous, { ref, ref: { name } }) => (Object.assign(Object.assign({}, previous), { [name]: getFieldValue(fields, ref) })), {});

var isEmptyObject = (value) => isObject(value) && !Object.keys(value).length;

var isSameError = (error, type, message) => isObject(error) && error.type === type && error.message === message;

function shouldUpdateWithError({ errors, name, error, validFields, fieldsWithValidation, }) {
    const isFieldValid = isEmptyObject(error);
    const isFormValid = isEmptyObject(errors);
    const currentFieldError = error[name];
    const existFieldError = errors[name];
    if ((isFieldValid && validFields.has(name)) ||
        (existFieldError && existFieldError.isManual)) {
        return false;
    }
    if (isFormValid !== isFieldValid ||
        (!isFormValid && !existFieldError) ||
        (isFieldValid && fieldsWithValidation.has(name) && !validFields.has(name))) {
        return true;
    }
    return (currentFieldError &&
        !isSameError(existFieldError, currentFieldError.type, currentFieldError.message));
}

var isRegex = (value) => value instanceof RegExp;

var getValueAndMessage = (validationData) => {
    const isPureObject = isObject(validationData) && !isRegex(validationData);
    return {
        value: isPureObject
            ? validationData.value
            : validationData,
        message: isPureObject
            ? validationData.message
            : '',
    };
};

var isString = (value) => typeof value === 'string';

var displayNativeError = (nativeValidation, ref, message) => {
    if (nativeValidation && isString(message)) {
        ref.setCustomValidity(message);
    }
};

var isFunction = (value) => typeof value === 'function';

var isBoolean = (value) => typeof value === 'boolean';

function getValidateError(result, ref, nativeError, type = 'validate') {
    const isStringValue = isString(result);
    if (isStringValue || (isBoolean(result) && !result)) {
        const message = isStringValue ? result : '';
        const error = {
            type,
            message,
            ref,
        };
        nativeError(message);
        return error;
    }
}

var appendErrors = (name, validateAllFieldCriteria, errors, type, message) => {
    if (!validateAllFieldCriteria) {
        return {};
    }
    const error = errors[name] || { types: {} };
    return Object.assign(Object.assign({}, error), { types: Object.assign(Object.assign({}, error.types), { [type]: message || true }) });
};

var validateField = async (fields, nativeValidation, validateAllFieldCriteria, { ref, ref: { type, value, name }, options, required, maxLength, minLength, min, max, pattern, validate, }) => {
    const error = {};
    const isRadio = isRadioInput(type);
    const isCheckBox = isCheckBoxInput(type);
    const isRadioOrCheckbox = isRadio || isCheckBox;
    const isEmpty = isEmptyString(value);
    const nativeError = displayNativeError.bind(null, nativeValidation, ref);
    const typedName = name;
    const appendErrorsCurry = appendErrors.bind(null, typedName, validateAllFieldCriteria, error);
    if (required &&
        ((!isRadio && !isCheckBox && (isEmpty || isNullOrUndefined(value))) ||
            (isCheckBox && !getCheckboxValue(options).isValid) ||
            (isRadio && !getRadioValue(options).isValid))) {
        const message = isString(required)
            ? required
            : getValueAndMessage(required).message;
        error[typedName] = Object.assign({ type: INPUT_VALIDATION_RULES.required, message, ref: isRadioOrCheckbox ? fields[typedName].options[0].ref : ref }, appendErrorsCurry(INPUT_VALIDATION_RULES.required, message));
        nativeError(message);
        if (!validateAllFieldCriteria) {
            return error;
        }
    }
    if (!isNullOrUndefined(min) || !isNullOrUndefined(max)) {
        let exceedMax;
        let exceedMin;
        const { value: maxValue, message: maxMessage } = getValueAndMessage(max);
        const { value: minValue, message: minMessage } = getValueAndMessage(min);
        if (type === 'number') {
            const valueNumber = parseFloat(value);
            if (!isNullOrUndefined(maxValue)) {
                exceedMax = valueNumber > maxValue;
            }
            if (!isNullOrUndefined(minValue)) {
                exceedMin = valueNumber < minValue;
            }
        }
        else {
            if (isString(maxValue)) {
                exceedMax = new Date(value) > new Date(maxValue);
            }
            if (isString(minValue)) {
                exceedMin = new Date(value) < new Date(minValue);
            }
        }
        if (exceedMax || exceedMin) {
            const message = exceedMax ? maxMessage : minMessage;
            error[typedName] = Object.assign({ type: exceedMax
                    ? INPUT_VALIDATION_RULES.max
                    : INPUT_VALIDATION_RULES.min, message,
                ref }, (exceedMax
                ? appendErrorsCurry(INPUT_VALIDATION_RULES.max, message)
                : appendErrorsCurry(INPUT_VALIDATION_RULES.min, message)));
            nativeError(message);
            if (!validateAllFieldCriteria) {
                return error;
            }
        }
    }
    if (isString(value) && !isEmpty && (maxLength || minLength)) {
        const { value: maxLengthValue, message: maxLengthMessage, } = getValueAndMessage(maxLength);
        const { value: minLengthValue, message: minLengthMessage, } = getValueAndMessage(minLength);
        const inputLength = value.toString().length;
        const exceedMax = maxLength && inputLength > maxLengthValue;
        const exceedMin = minLength && inputLength < minLengthValue;
        if (exceedMax || exceedMin) {
            const message = exceedMax ? maxLengthMessage : minLengthMessage;
            error[typedName] = Object.assign({ type: exceedMax
                    ? INPUT_VALIDATION_RULES.maxLength
                    : INPUT_VALIDATION_RULES.minLength, message,
                ref }, (exceedMax
                ? appendErrorsCurry(INPUT_VALIDATION_RULES.maxLength, message)
                : appendErrorsCurry(INPUT_VALIDATION_RULES.minLength, message)));
            nativeError(message);
            if (!validateAllFieldCriteria) {
                return error;
            }
        }
    }
    if (pattern && !isEmpty) {
        const { value: patternValue, message: patternMessage } = getValueAndMessage(pattern);
        if (isRegex(patternValue) && !patternValue.test(value)) {
            error[typedName] = Object.assign({ type: INPUT_VALIDATION_RULES.pattern, message: patternMessage, ref }, appendErrorsCurry(INPUT_VALIDATION_RULES.pattern, patternMessage));
            nativeError(patternMessage);
            if (!validateAllFieldCriteria) {
                return error;
            }
        }
    }
    if (validate) {
        const fieldValue = getFieldValue(fields, ref);
        const validateRef = isRadioOrCheckbox && options ? options[0].ref : ref;
        if (isFunction(validate)) {
            const result = await validate(fieldValue);
            const validateError = getValidateError(result, validateRef, nativeError);
            if (validateError) {
                error[typedName] = Object.assign(Object.assign({}, validateError), appendErrorsCurry(INPUT_VALIDATION_RULES.validate, validateError.message));
                if (!validateAllFieldCriteria) {
                    return error;
                }
            }
        }
        else if (isObject(validate)) {
            const validateFunctions = Object.entries(validate);
            const validationResult = await new Promise((resolve) => {
                validateFunctions.reduce(async (previous, [key, validate], index) => {
                    if ((!isEmptyObject(await previous) && !validateAllFieldCriteria) ||
                        !isFunction(validate)) {
                        return resolve(previous);
                    }
                    let result;
                    const validateResult = await validate(fieldValue);
                    const validateError = getValidateError(validateResult, validateRef, nativeError, key);
                    if (validateError) {
                        result = Object.assign(Object.assign({}, validateError), appendErrorsCurry(key, validateError.message));
                        if (validateAllFieldCriteria) {
                            error[typedName] = result;
                        }
                    }
                    else {
                        result = previous;
                    }
                    return validateFunctions.length - 1 === index
                        ? resolve(result)
                        : result;
                }, {});
            });
            if (!isEmptyObject(validationResult)) {
                error[typedName] = Object.assign({ ref: validateRef }, validationResult);
                if (!validateAllFieldCriteria) {
                    return error;
                }
            }
        }
    }
    if (nativeValidation) {
        ref.setCustomValidity('');
    }
    return error;
};

const parseErrorSchema = (error, validateAllFieldCriteria) => isArray(error.inner)
    ? error.inner.reduce((previous, { path, message, type }) => (Object.assign(Object.assign({}, previous), (previous[path] && validateAllFieldCriteria
        ? {
            [path]: appendErrors(path, validateAllFieldCriteria, previous, type, message),
        }
        : {
            [path]: Object.assign({ message,
                type }, (validateAllFieldCriteria
                ? {
                    types: { [type]: message || true },
                }
                : {})),
        }))), {})
    : {
        [error.path]: { message: error.message, type: error.type },
    };
async function validateWithSchema(validationSchema, validationSchemaOption, validateAllFieldCriteria, data) {
    try {
        return {
            result: await validationSchema.validate(data, validationSchemaOption),
            fieldErrors: {},
        };
    }
    catch (e) {
        return {
            result: {},
            fieldErrors: parseErrorSchema(e, validateAllFieldCriteria),
        };
    }
}

function attachNativeValidation(ref, rules) {
    Object.entries(rules).forEach(([key, value]) => {
        if (key === INPUT_VALIDATION_RULES.pattern && isRegex(value)) {
            ref[key] = value.source;
        }
        else {
            ref[key] = key === INPUT_VALIDATION_RULES.pattern || value;
        }
    });
}

var get = (obj, path, defaultValue) => {
    const result = path
        .split(/[,[\].]+?/)
        .filter(Boolean)
        .reduce((result, key) => (isNullOrUndefined(result) ? result : result[key]), obj);
    return isUndefined(result) || result === obj ? defaultValue : result;
};

var getDefaultValue = (defaultValues, name, defaultValue) => isUndefined(defaultValues[name])
    ? get(defaultValues, name, defaultValue)
    : defaultValues[name];

function flatArray(list) {
    return list.reduce((a, b) => a.concat(isArray(b) ? flatArray(b) : b), []);
}

var isPrimitive = (value) => isNullOrUndefined(value) || !isObjectType(value);

const getPath = (path, values) => isArray(values)
    ? values.map((item, index) => {
        const pathWithIndex = `${path}[${index}]`;
        return isPrimitive(item) ? pathWithIndex : getPath(pathWithIndex, item);
    })
    : Object.entries(values).map(([key, objectValue]) => {
        const pathWithKey = `${path}.${key}`;
        return isPrimitive(objectValue)
            ? pathWithKey
            : getPath(pathWithKey, objectValue);
    });
var getPath$1 = (parentPath, value) => flatArray(getPath(parentPath, value));

var assignWatchFields = (fieldValues, fieldName, watchFields, combinedDefaultValues) => {
    let value;
    if (isEmptyObject(fieldValues)) {
        value = undefined;
    }
    else if (!isUndefined(fieldValues[fieldName])) {
        watchFields.add(fieldName);
        value = fieldValues[fieldName];
    }
    else {
        value = get(combineFieldValues(fieldValues), fieldName);
        if (!isUndefined(value)) {
            getPath$1(fieldName, value).forEach(name => watchFields.add(name));
        }
    }
    return isUndefined(value)
        ? isObject(combinedDefaultValues)
            ? getDefaultValue(combinedDefaultValues, fieldName)
            : combinedDefaultValues
        : value;
};

var omitValidFields = (errorFields, validFieldNames) => Object.entries(errorFields).reduce((previous, [name, error]) => validFieldNames.some(validFieldName => validFieldName === name)
    ? previous
    : Object.assign(Object.assign({}, previous), { [name]: error }), {});

function onDomRemove(element, onDetachCallback) {
    const observer = new MutationObserver(() => {
        if (isDetached(element)) {
            observer.disconnect();
            onDetachCallback();
        }
    });
    observer.observe(window.document, {
        childList: true,
        subtree: true,
    });
    return observer;
}

/*! *****************************************************************************
Copyright (c) Microsoft Corporation. All rights reserved.
Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at http://www.apache.org/licenses/LICENSE-2.0

THIS CODE IS PROVIDED ON AN *AS IS* BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
KIND, EITHER EXPRESS OR IMPLIED, INCLUDING WITHOUT LIMITATION ANY IMPLIED
WARRANTIES OR CONDITIONS OF TITLE, FITNESS FOR A PARTICULAR PURPOSE,
MERCHANTABLITY OR NON-INFRINGEMENT.

See the Apache Version 2.0 License for specific language governing permissions
and limitations under the License.
***************************************************************************** */

function __rest(s, e) {
    var t = {};
    for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p) && e.indexOf(p) < 0)
        t[p] = s[p];
    if (s != null && typeof Object.getOwnPropertySymbols === "function")
        for (var i = 0, p = Object.getOwnPropertySymbols(s); i < p.length; i++) {
            if (e.indexOf(p[i]) < 0 && Object.prototype.propertyIsEnumerable.call(s, p[i]))
                t[p[i]] = s[p[i]];
        }
    return t;
}

const omitObject = (obj, key) => {
    const _a = key, omitted = obj[_a], rest = __rest(obj, [typeof _a === "symbol" ? _a : _a + ""]);
    return rest;
};

var modeChecker = (mode) => ({
    isOnSubmit: !mode || mode === VALIDATION_MODE.onSubmit,
    isOnBlur: mode === VALIDATION_MODE.onBlur,
    isOnChange: mode === VALIDATION_MODE.onChange,
});

const { useRef, useState, useCallback, useEffect } = React;
function useForm({ mode = VALIDATION_MODE.onSubmit, reValidateMode = VALIDATION_MODE.onChange, validationSchema, defaultValues = {}, nativeValidation = false, submitFocusError = true, validationSchemaOption = { abortEarly: false }, validateCriteriaMode, } = {}) {
    const fieldsRef = useRef({});
    const validateAllFieldCriteria = validateCriteriaMode === 'all';
    const errorsRef = useRef({});
    const touchedFieldsRef = useRef(new Set());
    const watchFieldsRef = useRef(new Set());
    const dirtyFieldsRef = useRef(new Set());
    const fieldsWithValidationRef = useRef(new Set());
    const validFieldsRef = useRef(new Set());
    const defaultInputValuesRef = useRef({});
    const defaultValuesRef = useRef(defaultValues);
    const isUnMount = useRef(false);
    const isWatchAllRef = useRef(false);
    const isSubmittedRef = useRef(false);
    const isDirtyRef = useRef(false);
    const submitCountRef = useRef(0);
    const isSubmittingRef = useRef(false);
    const validateAndUpdateStateRef = useRef();
    const [, _render] = useState();
    const { isOnBlur, isOnSubmit } = useRef(modeChecker(mode)).current;
    const isWindowUndefined = typeof window === UNDEFINED;
    const isWeb = typeof document !== UNDEFINED &&
        !isWindowUndefined &&
        !isUndefined(window.HTMLElement);
    const isProxyEnabled = !isWindowUndefined && 'Proxy' in window;
    const readFormState = useRef({
        dirty: !isProxyEnabled,
        isSubmitted: isOnSubmit,
        submitCount: !isProxyEnabled,
        touched: !isProxyEnabled,
        isSubmitting: !isProxyEnabled,
        isValid: !isProxyEnabled,
    });
    const { isOnBlur: isReValidateOnBlur, isOnSubmit: isReValidateOnSubmit, } = useRef(modeChecker(reValidateMode)).current;
    const validationSchemaOptionRef = useRef(validationSchemaOption);
    defaultValuesRef.current = defaultValues;
    const combineErrorsRef = (data) => (Object.assign(Object.assign({}, errorsRef.current), data));
    const render = useCallback(() => {
        if (!isUnMount.current) {
            _render({});
        }
    }, []);
    const validateFieldCurry = useCallback(validateField.bind(null, fieldsRef.current, nativeValidation, validateAllFieldCriteria), []);
    const validateWithSchemaCurry = useCallback(validateWithSchema.bind(null, validationSchema, validationSchemaOptionRef.current, validateAllFieldCriteria), [validationSchema]);
    const renderBaseOnError = useCallback((name, error, shouldRender) => {
        let reRender = shouldRender ||
            shouldUpdateWithError({
                errors: errorsRef.current,
                error,
                name,
                validFields: validFieldsRef.current,
                fieldsWithValidation: fieldsWithValidationRef.current,
            });
        if (isEmptyObject(error)) {
            if (fieldsWithValidationRef.current.has(name) || validationSchema) {
                validFieldsRef.current.add(name);
                reRender = reRender || errorsRef.current[name];
            }
            errorsRef.current = omitObject(errorsRef.current, name);
        }
        else {
            validFieldsRef.current.delete(name);
            reRender = reRender || !errorsRef.current[name];
        }
        errorsRef.current = combineErrorsRef(error);
        if (reRender) {
            render();
            return true;
        }
    }, [render, validationSchema]);
    const setFieldValue = useCallback((name, rawValue) => {
        const field = fieldsRef.current[name];
        if (!field) {
            return false;
        }
        const ref = field.ref;
        const { type } = ref;
        const options = field.options;
        const value = isWeb &&
            ref instanceof window.HTMLElement &&
            isNullOrUndefined(rawValue)
            ? ''
            : rawValue;
        if (isRadioInput(type) && options) {
            options.forEach(({ ref: radioRef }) => (radioRef.checked = radioRef.value === value));
        }
        else if (isMultipleSelect(type)) {
            [...ref.options].forEach(selectRef => (selectRef.selected = value.includes(selectRef.value)));
        }
        else if (isCheckBoxInput(type) && options) {
            options.length > 1
                ? options.forEach(({ ref: checkboxRef }) => (checkboxRef.checked = value.includes(checkboxRef.value)))
                : (options[0].ref.checked = !!value);
        }
        else {
            ref.value = value;
        }
        return type;
    }, [isWeb]);
    const setDirty = (name) => {
        if (!fieldsRef.current[name]) {
            return false;
        }
        const isDirty = defaultInputValuesRef.current[name] !==
            getFieldValue(fieldsRef.current, fieldsRef.current[name].ref);
        const isDirtyChanged = dirtyFieldsRef.current.has(name) !== isDirty;
        if (isDirty) {
            dirtyFieldsRef.current.add(name);
        }
        else {
            dirtyFieldsRef.current.delete(name);
        }
        isDirtyRef.current = !!dirtyFieldsRef.current.size;
        return isDirtyChanged && readFormState.current.dirty;
    };
    const setInternalValue = useCallback((name, value) => {
        setFieldValue(name, value);
        if (setDirty(name) ||
            (!touchedFieldsRef.current.has(name) && readFormState.current.touched)) {
            return !!touchedFieldsRef.current.add(name);
        }
    }, [setFieldValue]);
    const executeValidation = useCallback(async ({ name, value, }, shouldRender) => {
        const field = fieldsRef.current[name];
        if (!field) {
            return false;
        }
        if (!isUndefined(value)) {
            setInternalValue(name, value);
        }
        if (shouldRender) {
            render();
        }
        const error = await validateFieldCurry(field);
        renderBaseOnError(name, error);
        return isEmptyObject(error);
    }, [render, renderBaseOnError, setInternalValue, validateFieldCurry]);
    const executeSchemaValidation = useCallback(async (payload, shouldRender) => {
        const { fieldErrors } = await validateWithSchemaCurry(combineFieldValues(getFieldsValues(fieldsRef.current)));
        const isMultipleFields = isArray(payload);
        const names = isArray(payload)
            ? payload.map(({ name }) => name)
            : [payload.name];
        const validFieldNames = names.filter(name => !fieldErrors[name]);
        if (isMultipleFields) {
            errorsRef.current = omitValidFields(combineErrorsRef(Object.entries(fieldErrors)
                .filter(([key]) => names.includes(key))
                .reduce((previous, [name, error]) => (Object.assign(Object.assign({}, previous), { [name]: error })), {})), validFieldNames);
            render();
        }
        else {
            const fieldName = names[0];
            renderBaseOnError(fieldName, fieldErrors[fieldName]
                ? { [fieldName]: fieldErrors[fieldName] }
                : {}, shouldRender);
        }
        return isEmptyObject(errorsRef.current);
    }, [render, renderBaseOnError, validateWithSchemaCurry]);
    const triggerValidation = useCallback(async (payload, shouldRender) => {
        const fields = payload || Object.keys(fieldsRef.current).map(name => ({ name }));
        if (validationSchema) {
            return executeSchemaValidation(fields, shouldRender);
        }
        if (isArray(fields)) {
            const result = await Promise.all(fields.map(async (data) => await executeValidation(data, false)));
            render();
            return result.every(Boolean);
        }
        return await executeValidation(fields, shouldRender);
    }, [executeSchemaValidation, executeValidation, render, validationSchema]);
    const setValue = useCallback((name, value, shouldValidate) => {
        const shouldRender = setInternalValue(name, value) ||
            isWatchAllRef.current ||
            watchFieldsRef.current.has(name);
        if (shouldValidate) {
            return triggerValidation({ name }, shouldRender);
        }
        if (shouldRender) {
            render();
        }
        return;
    }, [render, setInternalValue, triggerValidation]);
    validateAndUpdateStateRef.current = validateAndUpdateStateRef.current
        ? validateAndUpdateStateRef.current
        : async ({ type, target }) => {
            const name = target ? target.name : '';
            const fields = fieldsRef.current;
            const errors = errorsRef.current;
            const field = fields[name];
            const currentError = errors[name];
            let error;
            if (!field) {
                return;
            }
            const isBlurEvent = type === EVENTS.BLUR;
            const shouldSkipValidation = (isOnSubmit && !isSubmittedRef.current) ||
                (isOnBlur && !isBlurEvent && !currentError) ||
                (isReValidateOnBlur && !isBlurEvent && currentError) ||
                (isReValidateOnSubmit && currentError);
            const shouldUpdateDirty = setDirty(name);
            let shouldUpdateState = isWatchAllRef.current ||
                watchFieldsRef.current.has(name) ||
                shouldUpdateDirty;
            if (isBlurEvent &&
                !touchedFieldsRef.current.has(name) &&
                readFormState.current.touched) {
                touchedFieldsRef.current.add(name);
                shouldUpdateState = true;
            }
            if (shouldSkipValidation) {
                return shouldUpdateState && render();
            }
            if (validationSchema) {
                const { fieldErrors } = await validateWithSchemaCurry(combineFieldValues(getFieldsValues(fields)));
                Object.keys(fieldErrors).forEach(name => validFieldsRef.current.delete(name));
                error = fieldErrors[name] ? { [name]: fieldErrors[name] } : {};
            }
            else {
                error = await validateFieldCurry(field);
            }
            if (!renderBaseOnError(name, error) && shouldUpdateState) {
                render();
            }
        };
    const resetFieldRef = useCallback((name) => {
        errorsRef.current = omitObject(errorsRef.current, name);
        fieldsRef.current = omitObject(fieldsRef.current, name);
        defaultInputValuesRef.current = omitObject(defaultInputValuesRef.current, name);
        [
            touchedFieldsRef,
            dirtyFieldsRef,
            fieldsWithValidationRef,
            validFieldsRef,
            watchFieldsRef,
        ].forEach(data => data.current.delete(name));
        if (readFormState.current.isValid || readFormState.current.touched) {
            render();
        }
    }, [render]);
    const removeEventListenerAndRef = useCallback((field, forceDelete) => {
        if (!field) {
            return;
        }
        findRemovedFieldAndRemoveListener(fieldsRef.current, validateAndUpdateStateRef.current, field, forceDelete);
        resetFieldRef(field.ref.name);
    }, [resetFieldRef]);
    function clearError(name) {
        if (isUndefined(name)) {
            errorsRef.current = {};
        }
        else {
            (isArray(name) ? name : [name]).forEach(fieldName => (errorsRef.current = omitObject(errorsRef.current, fieldName)));
        }
        render();
    }
    const setInternalError = ({ name, type, types, message, preventRender, }) => {
        const errors = errorsRef.current;
        if (!isSameError(errors[name], type, message)) {
            errors[name] = {
                type,
                types,
                message,
                ref: {},
                isManual: true,
            };
            if (!preventRender) {
                render();
            }
        }
    };
    function setError(name, type = '', message) {
        if (isString(name)) {
            setInternalError(Object.assign({ name }, (isObject(type)
                ? {
                    types: type,
                    type: '',
                }
                : {
                    type,
                    message,
                })));
        }
        else if (isArray(name)) {
            name.forEach(error => setInternalError(Object.assign(Object.assign({}, error), { preventRender: true })));
            render();
        }
    }
    function watch(fieldNames, defaultValue) {
        const combinedDefaultValues = isUndefined(defaultValue)
            ? isUndefined(defaultValues)
                ? {}
                : defaultValues
            : defaultValue;
        const fieldValues = getFieldsValues(fieldsRef.current);
        const watchFields = watchFieldsRef.current;
        if (isProxyEnabled) {
            readFormState.current.dirty = true;
        }
        if (isString(fieldNames)) {
            return assignWatchFields(fieldValues, fieldNames, watchFields, combinedDefaultValues);
        }
        if (isArray(fieldNames)) {
            return fieldNames.reduce((previous, name) => {
                let value = null;
                if (isEmptyObject(fieldsRef.current) &&
                    isObject(combinedDefaultValues)) {
                    value = getDefaultValue(combinedDefaultValues, name);
                }
                else {
                    value = assignWatchFields(fieldValues, name, watchFields, combinedDefaultValues);
                }
                return Object.assign(Object.assign({}, previous), { [name]: value });
            }, {});
        }
        isWatchAllRef.current = true;
        return ((!isEmptyObject(fieldValues) && fieldValues) ||
            defaultValue ||
            defaultValues);
    }
    function unregister(names) {
        if (!isEmptyObject(fieldsRef.current)) {
            (isArray(names) ? names : [names]).forEach(fieldName => removeEventListenerAndRef(fieldsRef.current[fieldName], true));
        }
    }
    function registerIntoFieldsRef(ref, validateOptions = {}) {
        if (!ref.name) {
            return console.warn('Missing name at', ref);
        }
        const { name, type, value } = ref;
        const fieldAttributes = Object.assign({ ref }, validateOptions);
        const fields = fieldsRef.current;
        const isRadioOrCheckbox = isRadioInput(type) || isCheckBoxInput(type);
        let currentField = fields[name];
        if (isRadioOrCheckbox
            ? currentField &&
                isArray(currentField.options) &&
                currentField.options.find(({ ref }) => value === ref.value)
            : currentField) {
            fields[name] = Object.assign(Object.assign({}, currentField), validateOptions);
            return;
        }
        if (type) {
            const mutationWatcher = onDomRemove(ref, () => removeEventListenerAndRef(fieldAttributes));
            if (isRadioOrCheckbox) {
                currentField = Object.assign({ options: [
                        ...((currentField && currentField.options) || []),
                        {
                            ref,
                            mutationWatcher,
                        },
                    ], ref: { type, name } }, validateOptions);
            }
            else {
                currentField = Object.assign(Object.assign({}, fieldAttributes), { mutationWatcher });
            }
        }
        else {
            currentField = fieldAttributes;
        }
        fields[name] = currentField;
        if (!isEmptyObject(defaultValuesRef.current)) {
            const defaultValue = getDefaultValue(defaultValuesRef.current, name);
            if (!isUndefined(defaultValue)) {
                setFieldValue(name, defaultValue);
            }
        }
        if (!isEmptyObject(validateOptions)) {
            fieldsWithValidationRef.current.add(name);
            const shouldRender = () => {
                if (validFieldsRef.current.size === fieldsWithValidationRef.current.size) {
                    render();
                }
            };
            if (!isOnSubmit && readFormState.current.isValid) {
                if (validationSchema) {
                    validateWithSchemaCurry(combineFieldValues(getFieldsValues(fields))).then(({ fieldErrors }) => {
                        if (fieldErrors[name]) {
                            validFieldsRef.current.add(name);
                        }
                        shouldRender();
                    });
                }
                else {
                    validateFieldCurry(currentField).then(error => {
                        if (isEmptyObject(error)) {
                            validFieldsRef.current.add(name);
                        }
                        shouldRender();
                    });
                }
            }
        }
        if (!defaultInputValuesRef.current[name]) {
            defaultInputValuesRef.current[name] = getFieldValue(fields, currentField.ref);
        }
        if (!type) {
            return;
        }
        const fieldToAttachListener = isRadioOrCheckbox && currentField.options
            ? currentField.options[currentField.options.length - 1]
            : currentField;
        if (nativeValidation && validateOptions) {
            attachNativeValidation(ref, validateOptions);
        }
        else {
            attachEventListeners({
                field: fieldToAttachListener,
                isRadioOrCheckbox,
                validateAndStateUpdate: validateAndUpdateStateRef.current,
            });
        }
    }
    function register(refOrValidateRule, validationOptions) {
        if (isWindowUndefined || !refOrValidateRule) {
            return;
        }
        if (validationOptions && isString(validationOptions.name)) {
            registerIntoFieldsRef({ name: validationOptions.name }, validationOptions);
            return;
        }
        if (isObject(refOrValidateRule) && 'name' in refOrValidateRule) {
            registerIntoFieldsRef(refOrValidateRule, validationOptions);
            return;
        }
        return (ref) => ref && registerIntoFieldsRef(ref, refOrValidateRule);
    }
    const handleSubmit = useCallback((callback) => async (e) => {
        if (e) {
            e.preventDefault();
            e.persist();
        }
        let fieldErrors;
        let fieldValues;
        const fields = fieldsRef.current;
        if (readFormState.current.isSubmitting) {
            isSubmittingRef.current = true;
            render();
        }
        try {
            if (validationSchema) {
                fieldValues = getFieldsValues(fields);
                const output = await validateWithSchemaCurry(combineFieldValues(fieldValues));
                errorsRef.current = output.fieldErrors;
                fieldErrors = output.fieldErrors;
                fieldValues = output.result;
            }
            else {
                const { errors, values, } = await Object.values(fields).reduce(async (previous, field) => {
                    if (!field) {
                        return previous;
                    }
                    const resolvedPrevious = await previous;
                    const { ref, ref: { name }, } = field;
                    if (!fields[name]) {
                        return Promise.resolve(resolvedPrevious);
                    }
                    const fieldError = await validateFieldCurry(field);
                    if (fieldError[name]) {
                        resolvedPrevious.errors = Object.assign(Object.assign({}, resolvedPrevious.errors), fieldError);
                        validFieldsRef.current.delete(name);
                        return Promise.resolve(resolvedPrevious);
                    }
                    if (fieldsWithValidationRef.current.has(name)) {
                        validFieldsRef.current.add(name);
                    }
                    resolvedPrevious.values[name] = getFieldValue(fields, ref);
                    return Promise.resolve(resolvedPrevious);
                }, Promise.resolve({
                    errors: {},
                    values: {},
                }));
                fieldErrors = errors;
                fieldValues = values;
            }
            if (isEmptyObject(fieldErrors)) {
                errorsRef.current = {};
                await callback(combineFieldValues(fieldValues), e);
            }
            else {
                if (submitFocusError) {
                    Object.keys(fieldErrors).reduce((previous, current) => {
                        const field = fields[current];
                        if (field && previous) {
                            if (field.ref.focus) {
                                field.ref.focus();
                                return false;
                            }
                            else if (field.options) {
                                field.options[0].ref.focus();
                                return false;
                            }
                        }
                        return previous;
                    }, true);
                }
                errorsRef.current = fieldErrors;
            }
        }
        finally {
            isSubmittedRef.current = true;
            isSubmittingRef.current = false;
            submitCountRef.current = submitCountRef.current + 1;
            render();
        }
    }, [
        render,
        submitFocusError,
        validateFieldCurry,
        validateWithSchemaCurry,
        validationSchema,
    ]);
    const resetRefs = () => {
        errorsRef.current = {};
        defaultInputValuesRef.current = {};
        touchedFieldsRef.current = new Set();
        watchFieldsRef.current = new Set();
        dirtyFieldsRef.current = new Set();
        validFieldsRef.current = new Set();
        isWatchAllRef.current = false;
        isSubmittedRef.current = false;
        isDirtyRef.current = false;
        submitCountRef.current = 0;
    };
    const reset = useCallback((values) => {
        const fieldsKeyValue = Object.entries(fieldsRef.current);
        for (const [, value] of fieldsKeyValue) {
            if (value && value.ref && value.ref.closest) {
                try {
                    value.ref.closest('form').reset();
                    break;
                }
                catch (_a) { }
            }
        }
        resetRefs();
        if (values) {
            fieldsKeyValue.forEach(([key]) => setFieldValue(key, getDefaultValue(values, key)));
            defaultInputValuesRef.current = Object.assign({}, values);
            if (readFormState.current.isValid) {
                triggerValidation();
            }
        }
        render();
    }, [render, setFieldValue, triggerValidation]);
    const getValues = useCallback((payload) => {
        const fieldValues = getFieldsValues(fieldsRef.current);
        const outputValues = isEmptyObject(fieldValues)
            ? defaultValues
            : fieldValues;
        return payload && payload.nest
            ? combineFieldValues(outputValues)
            : outputValues;
    }, [defaultValues]);
    useEffect(() => () => {
        isUnMount.current = true;
        fieldsRef.current &&
            Object.values(fieldsRef.current).forEach((field) => removeEventListenerAndRef(field, true));
    }, [removeEventListenerAndRef]);
    const formState = Object.assign({ dirty: isDirtyRef.current, isSubmitted: isSubmittedRef.current, submitCount: submitCountRef.current, touched: [...touchedFieldsRef.current], isSubmitting: isSubmittingRef.current }, (isOnSubmit
        ? {
            isValid: isSubmittedRef.current && isEmptyObject(errorsRef.current),
        }
        : {
            isValid: fieldsWithValidationRef.current.size
                ? !isEmptyObject(fieldsRef.current) &&
                    validFieldsRef.current.size >=
                        fieldsWithValidationRef.current.size &&
                    isEmptyObject(errorsRef.current)
                : !isEmptyObject(fieldsRef.current),
        }));
    return {
        register: useCallback(register, []),
        unregister: useCallback(unregister, [removeEventListenerAndRef]),
        handleSubmit,
        watch,
        reset,
        clearError: useCallback(clearError, []),
        setError: useCallback(setError, []),
        setValue,
        triggerValidation,
        getValues,
        errors: errorsRef.current,
        formState: isProxyEnabled
            ? new Proxy(formState, {
                get: (obj, prop) => {
                    if (prop in obj) {
                        readFormState.current[prop] = true;
                        return obj[prop];
                    }
                    return {};
                },
            })
            : formState,
    };
}

const FormGlobalContext = React.createContext(null);
function useFormContext() {
    return React.useContext(FormGlobalContext);
}
function FormContext(props) {
    const { children, formState, errors } = props, restMethods = __rest(props, ["children", "formState", "errors"]);
    const restRef = React.useRef(restMethods);
    return (React.createElement(FormGlobalContext.Provider, { value: Object.assign(Object.assign({}, restRef.current), { formState, errors }) }, children));
}

export { FormContext, useForm as default, useFormContext }

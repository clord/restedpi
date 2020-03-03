import { useEffect, useLayoutEffect, useReducer, useRef } from '/react/';

export default function create(createState) {
    let state;
    let subscribers = [];
    let subscriberCount = 0;
    const setState = partial => {
        const partialState = typeof partial === 'function' ? partial(state) : partial;
        if (partialState !== state) {
            state = Object.assign({}, state, partialState);
            // Reset subscriberCount because we will be removing holes from the
            // subscribers array and changing the length which should be the same as
            // subscriberCount.
            subscriberCount = 0;
            // Create a dense array by removing holes from the subscribers array.
            // Holes are not iterated by Array.prototype.filter.
            subscribers = subscribers.filter(subscriber => {
                subscriber.index = subscriberCount++;
                return true;
            });
            // Call all subscribers only after the subscribers array has been changed
            // to a dense array. Subscriber callbacks cannot be called above in
            // subscribers.filter because the callbacks can cause a synchronous
            // increment of subscriberCount if not batched.
            subscribers.forEach(subscriber => subscriber.callback());
        }
    };
    const getState = () => state;
    const getSubscriber = (listener, selector = getState, equalityFn = Object.is) => ({
        callback: () => { },
        currentSlice: selector(state),
        equalityFn,
        errored: false,
        index: subscriberCount++,
        listener,
        selector,
    });
    const subscribe = (subscriber) => {
        subscriber.callback = () => {
            // Selector or equality function could throw but we don't want to stop
            // the listener from being called.
            // https://github.com/react-spring/zustand/pull/37
            try {
                const newStateSlice = subscriber.selector(state);
                if (!subscriber.equalityFn(subscriber.currentSlice, newStateSlice)) {
                    subscriber.listener((subscriber.currentSlice = newStateSlice));
                }
            }
            catch (error) {
                subscriber.errored = true;
                subscriber.listener(null, error);
            }
        };
        // subscriber.index is set during the render phase in order to store the
        // subscibers in a top-down order. The subscribers array will become a
        // sparse array when an index is skipped (due to an interrupted render) or
        // a component unmounts and the subscriber is deleted. It's converted back
        // to a dense array in setState.
        subscribers[subscriber.index] = subscriber;
        // Delete creates a hole and preserves the array length. If we used
        // Array.prototype.splice, subscribers with a greater subscriber.index
        // would no longer match their actual index in subscribers.
        return () => delete subscribers[subscriber.index];
    };
    const apiSubscribe = (listener, selector, equalityFn) => subscribe(getSubscriber(listener, selector, equalityFn));
    const destroy = () => (subscribers = []);
    const useStore = (selector = getState, equalityFn = Object.is) => {
        const forceUpdate = useReducer(c => c + 1, 0)[1];
        const subscriberRef = useRef();
        if (!subscriberRef.current) {
            subscriberRef.current = getSubscriber(forceUpdate, selector, equalityFn);
        }
        const subscriber = subscriberRef.current;
        let newStateSlice;
        let hasNewStateSlice = false;
        // The selector or equalityFn need to be called during the render phase if
        // they change. We also want legitimate errors to be visible so we re-run
        // them if they errored in the subscriber.
        if (subscriber.selector !== selector ||
            subscriber.equalityFn !== equalityFn ||
            subscriber.errored) {
            // Using local variables to avoid mutations in the render phase.
            newStateSlice = selector(state);
            hasNewStateSlice = !equalityFn(subscriber.currentSlice, newStateSlice);
        }
        // Syncing changes in useEffect.
        useLayoutEffect(() => {
            if (hasNewStateSlice) {
                subscriber.currentSlice = newStateSlice;
            }
            subscriber.selector = selector;
            subscriber.equalityFn = equalityFn;
            subscriber.errored = false;
        });
        useLayoutEffect(() => subscribe(subscriber), []);
        return hasNewStateSlice
            ? newStateSlice
            : subscriber.currentSlice;
    };
    const api = { setState, getState, subscribe: apiSubscribe, destroy };
    state = createState(setState, getState, api);
    return [useStore, api];
}
export { create };


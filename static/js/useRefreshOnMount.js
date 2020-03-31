import { useAppStore } from '/js/hooks/useApp.js';
import { useRoute } from '/js/depend/wouter/';
import { useEffect } from '/react/';

export function useRefreshOnMount(path, accessor, props = []) {
  const getter = useAppStore(accessor);
  const [match, params] = useRoute(path);
  useEffect(() => {
    if (match) {
      if (props.length === 0) {
        getter();
      } else {
        for (const prop of props) {
          if (params == null || params[prop] == null) {
            return;
          }
        }
        getter(params);
      }
    }
  }, []);
}

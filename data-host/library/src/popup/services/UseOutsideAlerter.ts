import React, { useEffect } from 'react';

export const useOutsideAlerter = (
  ref: React.RefObject<HTMLDivElement>,
  onClickOutsideCallback: () => void
) => {
  useEffect(() => {
    function handleClickOutside(this: Document, event: MouseEvent): void {
      if (ref.current !== null && event.target !== null) {
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        if (!ref.current.contains(event.target)) onClickOutsideCallback();
      }
    }

    // Bind the event listener
    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      // Unbind the event listener on clean up
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [onClickOutsideCallback, ref]);
};

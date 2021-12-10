import { createContext, useContext, useRef, useState } from "react";
import styled from "styled-components";

export type NodeStack = React.ReactNode[];

export class Updater {
  constructor(
    public stack: NodeStack,
    private readonly reactSetStack: (stack: NodeStack) => void,
  ) {}

  private setStack(stack: NodeStack) {
    this.stack = stack;
    this.reactSetStack(this.stack);
  }

  push(node: React.ReactNode) {
    this.setStack([...this.stack, node]);
  }

  pop() {
    this.setStack(this.stack.slice(0, this.stack.length - 1));
  }
}

export const ActivityStackContext = createContext<{
  stack: NodeStack;
  updater: Updater;
}>({
  stack: [],
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  updater: new Updater([], () => {}),
});

export function ActivityStack({ children }: { children: React.ReactNode }) {
  const { stack, updater } = useContext(ActivityStackContext);

  const lastItem = stack[stack.length - 1];
  const content =
    lastItem == null ? (
      children
    ) : (
      <CloseActivity updater={updater}>{lastItem}</CloseActivity>
    );

  return <>{content}</>;
}

export function ActivityStackProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const [stack, setStack] = useState<React.ReactNode[]>([]);
  // We need to keep the same instance of the Updater so callbacks in Nodes
  // pushed to the stack can pop from the correct instance.
  const { current: updater } = useRef(new Updater(stack, setStack));

  return (
    <ActivityStackContext.Provider value={{ stack, updater }}>
      {children}
    </ActivityStackContext.Provider>
  );
}

const CloseContainer = styled.div`
  position: relative;
`;

const CloseButton = styled.button`
  position: absolute;
  top: var(--s-12);
  right: var(--s-12);

  height: 32px;
  width: 32px;

  color: var(--c-white);
  background: var(--c-orange);
  border-radius: 50%;
  padding: var(--s-6);
  font-size: large;
`;

function CloseActivity({
  updater,
  children,
}: {
  updater: Updater;
  children: React.ReactNode;
}) {
  return (
    <CloseContainer>
      <CloseButton aria-label="Close" onClick={() => updater.pop()}>
        ✕
      </CloseButton>

      {children}
    </CloseContainer>
  );
}

import React, { useState } from "react";

import TabButton from "@popup/components/common/TabButton";

import styled from "styled-components";
import { useLocation } from "react-router";

const RootContainer = styled.div``;
const TabsButtonsContainer = styled.div`
  display: flex;
  flex-direction: row;
`;
const TabContainer = styled.div``;

interface Tab {
  id: string;
  label: string;
  props: any;
  component: (props: any) => JSX.Element;
  disabled?: boolean;
}

interface TabsProps {
  tabs: Tab[];
  activeTab?: string;
  onTabChange?: (id: string) => void;
}

function Tabs(props: TabsProps & React.HTMLAttributes<HTMLDivElement>) {
  const { tabs, activeTab: activeTabProp, onTabChange, ...otherProps } = props;

  const [userSelected, setUserSelected] = useState<string | undefined>();

  const getTab = (id: string) => tabs.find((tab) => tab.id === id);

  const query = new URLSearchParams(useLocation().search);
  const selected = [
    userSelected,
    activeTabProp,
    query.get("activeTab"),
    tabs[0].id,
  ].find(
    (tab) => tab != null && getTab(tab) != null && !getTab(tab)!.disabled,
  )!;

  const changeTab = (id: string) => {
    setUserSelected(id);
    if (onTabChange) onTabChange(id);
  };

  const selectedTab = getTab(selected) || tabs[0];

  return (
    <RootContainer {...otherProps}>
      <TabsButtonsContainer>
        {tabs.map((tab, index) => (
          <TabButton
            key={tab.id}
            index={index}
            total={tabs.length}
            onClick={() => changeTab(tab.id)}
            selected={tab.id === selected}
            label={tab.label}
            disabled={tab.disabled}
          />
        ))}
      </TabsButtonsContainer>
      <TabContainer>
        <selectedTab.component {...selectedTab.props} />
      </TabContainer>
    </RootContainer>
  );
}

export default Tabs;

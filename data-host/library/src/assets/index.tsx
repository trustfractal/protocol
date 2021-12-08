import { useOutsideAlerter } from "@popup/hooks/useOutsideAlerter";
import { Utils } from "@trustfractal/sdk";
import React, { useState } from "react";

import styled, { css } from "styled-components";
import Icon, { IconNames } from "../Icon";

const MenuButton = styled.button<{ active: boolean }>`
  position: absolute;
  top: var(--s-12);
  right: var(--s-12);
  width: 48px;
  height: 48px;
  border-radius: 50%;
  cursor: pointer;
  background: ${(props) =>
    props.active ? "var(--c-orange)" : "var(--c-transparent)"};
  transition-color: background 0.3s ease-in-out;
`;

const MenuContainer = styled.div<{ active: boolean }>`
  position: absolute;
  display: flex;
  flex-direction: column;
  width: 292px;
  right: var(--s-12);
  top: calc(48px + var(--s-12) + var(--s-12));
  border-radius: var(--s-12);
  background: var(--c-orange);
  transform: ${(props) =>
    props.active ? "translateX(0)" : "translateX(calc(100% + var(--s-12)))"};
  transition: transform 0.3s ease-in-out;
  z-index: 2;
`;

const MenuOverlay = styled.div<{ active: boolean }>`
  position: absolute;
  width: 292px;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: var(--c-dark-blue);
  opacity: ${(props) => (props.active ? "0.2" : "0")};
  transition: opacity 0.3s ease-in-out;
  z-index: 1;
  pointer-events: none;
`;

const MenuLink = styled.button<{ disabled?: boolean }>`
  display: flex;
  align-items: center;
  padding: var(--s-24) var(--s-20);
  background: none;
  color: var(--c-white);
  cursor: pointer;

  :not(:first-child)  {
    border-top: 1px solid rgba(255, 255, 255);
  }

  ${(props) =>
    props.disabled &&
    css`
      opacity: 0.6;
      cursor: default;
    `}
`;

const SubMenuLink = styled.button<{ disabled?: boolean }>`
  display: flex;
  align-items: center;
  padding: var(--s-24) var(--s-20);
  background: none;
  color: var(--c-white);
  cursor: pointer;

  :not(:first-child)  {
    border-top: 1px solid rgba(255, 255, 255, 0.2);
  }

  ${(props) =>
    props.disabled &&
    css`
      opacity: 0.6;
      cursor: default;
    `}
`;

const IconContainer = styled.div`
  margin-right: var(--s-12);
`;

type Item = {
  label: string;
  icon: string;
  onClick: () => void;
  disabled?: boolean;
};

type MenuItem = Item | Item[];

type MenuProps = {
  items: MenuItem[];
};

function Menu(
  props: MenuProps & React.ButtonHTMLAttributes<HTMLButtonElement>,
) {
  const { items } = props;

  const [menuOpen, setMenuOpen] = useState(false);

  const closeMenu = () => setMenuOpen(false);
  const menuRef = React.createRef<HTMLDivElement>();
  useOutsideAlerter(menuRef, closeMenu);

  return (
    <>
      <MenuButton active={menuOpen} onClick={() => setMenuOpen(!menuOpen)}>
        {menuOpen && <Icon name={IconNames.MENU_ACTIVE} />}
        {!menuOpen && <Icon name={IconNames.MENU_INACTIVE} />}
      </MenuButton>
      <MenuContainer active={menuOpen} ref={menuRef}>
        {items.map((item: Item | Item[]) => {
          if (Utils.isArray(item)) {
            return (item as Item[]).map((subItem: Item) => {
              const { label, icon, onClick, disabled } = subItem;

              return (
                <SubMenuLink
                  key={label}
                  onClick={() => {
                    closeMenu();
                    onClick();
                  }}
                  disabled={disabled}
                >
                  <IconContainer>
                    <Icon name={icon} />
                  </IconContainer>
                  {label}
                </SubMenuLink>
              );
            });
          }

          const { label, icon, onClick, disabled } = item as Item;

          return (
            <MenuLink
              key={label}
              onClick={() => {
                closeMenu();
                onClick();
              }}
              disabled={disabled}
            >
              <IconContainer>
                <Icon name={icon} />
              </IconContainer>
              {label}
            </MenuLink>
          );
        })}
      </MenuContainer>
      <MenuOverlay active={menuOpen} />
    </>
  );
}

export default Menu;

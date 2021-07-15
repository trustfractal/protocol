export const Pallette = {
  Bone: '#EAE0CC',
  Bittersweet: '#FF6F59',
  EerieBlack: '#222222',
  Platinum: '#EAEAEA',
  Zemp: '#43AA8B',
};

export const Sizes = {
  xs: '6px',
  s: '8px',
  m: '11px',
  l: '15px',
  xl: '20px',
};

export type Size = 'xs' | 's' | 'm' | 'l' | 'xl';
export type Color = 'Bone' | 'Bittersweet' | 'EerieBlack' | 'Platinum' | 'Zemp';

const Theme = {
  Pallette,
  Sizes,
};

export default Theme;

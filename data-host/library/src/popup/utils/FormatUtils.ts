import numeral from 'numeral';

export function formatBalance(number: number, decimals = 4): string {
  // check if number is gte one million
  if (number >= 10 ** 6) {
    return numeral(number).format(`0.${'0'.repeat(decimals)}e+0`);
  }

  return numeral(number).format(`0[.]${'0'.repeat(decimals)}`);
}

function capitalize(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

export function fromSnackCase(value: string): string {
  return capitalize(value.replaceAll('_', ' '));
}

export function formatFloat(value: number, maxDigits: number) {
  return value.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: maxDigits,
  });
}

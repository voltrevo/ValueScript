declare const Debug: {
  log: (...args: unknown[]) => undefined;
};

export default function main() {
  let year = 1900;
  let month = 0;
  let monthLen = getMonthLen(year, month);
  let day = 0;
  let weekday = 1; // Monday (Sunday=0)

  let count = 0;

  while (true) {
    if (year >= 1901 && day === 0 && weekday === 0) {
      count++;
    }

    day++;
    weekday = (weekday + 1) % 7;

    if (day === monthLen) {
      day = 0;
      month++;

      if (month === 12) {
        month = 0;
        year++;

        if (year === 2001) {
          return count;
        }
      }

      monthLen = getMonthLen(year, month);
    }
  }
}

function getMonthLen(year: number, month: number) {
  if (month === 1) { // February due to starting at 0
    if (year % 4 !== 0) {
      return 28;
    }

    if (year % 100 !== 0) {
      return 29;
    }

    if (year % 400 !== 0) {
      return 28;
    }

    return 29;
  }

  return [
    31,
    28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ][month];
}

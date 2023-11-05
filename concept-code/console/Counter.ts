import type ConsoleApp from "./ConsoleApp.ts";
import type { RenderInfo } from "./ConsoleApp.ts";

type View = {
  offset: number;
} | undefined;

export default class Counter implements ConsoleApp<Counter, View> {
  value = 0;

  createView(): View {
    return { offset: 0 };
  }

  render = function (
    this: { db: Counter; view: View },
    { screenWidth, screenHeight }: RenderInfo,
  ) {
    if (this.view === undefined) {
      return undefined;
    }

    let wCenter = Math.floor(screenWidth / 2);
    let hCenter = Math.floor(screenHeight / 2);

    let lines = [];

    for (let i = 0; i < hCenter; i++) {
      lines.push("");
    }

    lines.push(" ".repeat(wCenter + this.view.offset) + this.db.value);

    return lines;
  };

  onKeyDown = function (this: { db: Counter; view: View }, key: string) {
    if (this.view === undefined) {
      return;
    }

    switch (key) {
      case "q": {
        this.view = undefined;
        break;
      }

      case "ArrowLeft": {
        this.view.offset--;
        break;
      }

      case "ArrowRight": {
        this.view.offset++;
        break;
      }

      case "ArrowUp": {
        this.db.value++;
        break;
      }

      case "ArrowDown": {
        this.db.value--;
        break;
      }
    }
  };
}

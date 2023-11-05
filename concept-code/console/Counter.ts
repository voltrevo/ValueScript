import type ConsoleApp from "./ConsoleApp.ts";
import type { RenderInfo } from "./ConsoleApp.ts";

type View = {
  offset: number;
};

export default class ConsoleAppDemo
  implements ConsoleApp<ConsoleAppDemo, View> {
  value = 0;

  createView(): View {
    return { offset: 0 };
  }

  render = function (
    this: { db: ConsoleAppDemo; view: View },
    { screenWidth, screenHeight }: RenderInfo,
  ) {
    return `${
      " ".repeat(this.view.offset)
    }${this.db.value}\n${screenWidth}x${screenHeight}`;
  };

  onKeyDown = function (this: { db: ConsoleAppDemo; view: View }, key: string) {
    switch (key) {
      case "ArrowLeft": {
        this.view.offset = Math.max(0, this.view.offset - 1);
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

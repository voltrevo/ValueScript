export type RenderInfo = {
  screenWidth: number;
  screenHeight: number;
};

type ConsoleApp<Db, View> = {
  createView(): View;

  render: (
    this: { db: Db; view: View },
    info: RenderInfo,
  ) => string;

  onKeyDown: (this: { db: Db; view: View }, key: string) => void;
};

export default ConsoleApp;

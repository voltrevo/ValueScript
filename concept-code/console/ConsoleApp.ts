type ConsoleApp<Db, View> = {
  createView(): View;
  render: (this: { db: Db, view: View }) => string;
  onKeyDown: (this: { db: Db, view: View }, key: string) => void;
};

export default ConsoleApp;

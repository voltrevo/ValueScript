import nil from './helpers/nil';

export default class FileSystem {
  list: string[];
  files: Record<string, string | nil> = {};

  constructor(public defaults: Record<string, string | nil> = {}) {
    const storedList: string[] | null = JSON.parse(
      localStorage.getItem('fs-list') ?? 'null',
    );

    if (storedList !== null) {
      this.list = storedList;
    } else {
      this.list = Object.keys(defaults);
    }

    for (const file of this.list) {
      const storedFile: string | null = localStorage.getItem(`fs-${file}`);
      this.files[file] = storedFile ?? defaults[file];
    }
  }

  read(file: string): string | nil {
    return this.files[file];
  }

  write(file: string, content: string | nil): void {
    if (content === nil) {
      this.list = this.list.filter((f) => f !== file);
      localStorage.setItem('fs-list', JSON.stringify(this.list));
      localStorage.removeItem(`fs-${file}`);
      delete this.files[file];
    } else {
      if (!this.list.includes(file)) {
        this.list.push(file);
        localStorage.setItem('fs-list', JSON.stringify(this.list));
      }
  
      this.files[file] = content;

      if (this.defaults[file] === content) {
        localStorage.removeItem(`fs-${file}`);
      } else {
        localStorage.setItem(`fs-${file}`, content);
      }
    }
  }
}

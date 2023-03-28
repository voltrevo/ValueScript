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

    const missingFiles: string[] = [];

    for (const file of this.list) {
      const storedFile: string | null = localStorage.getItem(`fs-${file}`);
      const content = storedFile ?? defaults[file];

      if (content === nil) {
        missingFiles.push(file);
        continue;
      }

      this.files[file] = content;
    }

    for (const file of missingFiles) {
      this.write(file, nil);
    }

    for (const file of Object.keys(defaults)) {
      if (!this.list.includes(file)) {
        this.write(file, defaults[file]);
      }
    }
  }

  read(file: string): string | nil {
    return this.files[file];
  }

  write(file: string, content: string | nil, afterFile?: string): void {
    if (content === nil) {
      this.list = this.list.filter((f) => f !== file);
      localStorage.setItem('fs-list', JSON.stringify(this.list));
      localStorage.removeItem(`fs-${file}`);
      delete this.files[file];
    } else {
      if (!this.list.includes(file)) {
        if (afterFile === nil) {
          this.list.push(file);
        } else {
          const index = this.list.indexOf(afterFile);
          this.list.splice(index + 1, 0, file);
        }

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

  rename(file: string, newFile: string): void {
    const content = this.read(file);

    if (content === nil) {
      return;
    }

    this.write(newFile, content, file);
    this.write(file, nil);
  }
}
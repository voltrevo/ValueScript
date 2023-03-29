import { defaultFiles, orderedFiles } from './files';
import nil from './helpers/nil';

export default class FileSystem {
  list: string[];
  files: Record<string, string | nil> = {};

  constructor() {
    const storedList: string[] = JSON.parse(
      localStorage.getItem('fs-list') ?? '[]',
    );

    if (orderedFiles.find(f => !storedList.includes(f)) !== undefined) {
      this.list = [
        ...orderedFiles,
        ...storedList.filter(f => !orderedFiles.includes(f)),
      ];
    } else {
      this.list = [...storedList];
    }

    this.list = [
      ...this.list,
      ...Object.keys(defaultFiles).filter(f => !this.list.includes(f)),
    ];

    localStorage.setItem('fs-list', JSON.stringify(this.list));

    for (const file of [...this.list]) {
      const storedFile: string | null = localStorage.getItem(`fs-${file}`);
      const content = storedFile ?? defaultFiles[file];
      this.write(file, content);
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

      if (defaultFiles[file] === content) {
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

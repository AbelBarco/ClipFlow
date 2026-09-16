type ColorFormat = 'hex' | 'rgb' | 'hsl' | 'css';

class ColorStore {
  activeFormat: ColorFormat = $state('hex');
  recentColors: string[] = $state([]);

  setFormat(format: ColorFormat): void {
    this.activeFormat = format;
  }

  addRecentColor(color: string): void {
    this.recentColors = [color, ...this.recentColors.filter(c => c !== color)].slice(0, 20);
  }

  getFormattedColor(color: string): string {
    return color;
  }
}

export const colorStore = new ColorStore();
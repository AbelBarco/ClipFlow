import type { Transformer, TransformerType } from '../types/transformers.types';
import { TRANSFORMERS } from '../types/transformers.types';

class TransformersStore {
  availableTransformers: Transformer[] = $state(TRANSFORMERS);
  recentTransformers: TransformerType[] = $state([]);
  activeTransformer: TransformerType | null = $state(null);

  getTransformersByCategory(category: Transformer['category']): Transformer[] {
    return this.availableTransformers.filter(t => t.category === category);
  }

  addRecent(transformer: TransformerType): void {
    this.recentTransformers = [
      transformer,
      ...this.recentTransformers.filter(t => t !== transformer)
    ].slice(0, 5);
  }

  setActive(transformer: TransformerType | null): void {
    this.activeTransformer = transformer;
  }
}

export const transformersStore = new TransformersStore();
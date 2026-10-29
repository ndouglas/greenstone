"""Rosenblatt's perceptron: one neuron, a line, and a rule for nudging it."""

import numpy as np


class Perceptron:
    def __init__(self, n_inputs: int):
        self.w = np.zeros(n_inputs)
        self.b = 0.0

    def predict(self, x: np.ndarray) -> np.ndarray:
        """Yes (1) on the side of the line the weights point to, else no (0)."""
        return (x @ self.w + self.b > 0).astype(int)

    def fit(self, x: np.ndarray, y: np.ndarray, epochs: int, rate: float, seed: int) -> list[int]:
        """Go through the examples, nudging the line after each mistake.
        Returns the number of mistakes in each pass."""
        rng = np.random.default_rng(seed)
        mistakes = []
        for _ in range(epochs):
            wrong = 0
            for i in rng.permutation(len(y)):
                error = y[i] - self.predict(x[i : i + 1])[0]
                if error:
                    self.w += rate * error * x[i]
                    self.b += rate * error
                    wrong += 1
            mistakes.append(wrong)
        return mistakes

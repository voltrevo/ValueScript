#include <iostream>

int fib(int n);

int main() {
  std::cout << fib(38) << std::endl;
  return 0;
}

int fib(int n) {
  if (n < 2) {
    return n;
  }

  return fib(n - 1) + fib(n - 2);
}

import argparse
from src import OOGA


def __parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("filename", type=str, help="Filename of the puzzle to solve")
    args = parser.parse_args()
    return args


def main():
    args = __parse_args()
    print(args)
    print(OOGA)


if __name__ == "__main__":
    main()

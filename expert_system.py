import argparse
from src import parse_file


def __parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "filename", type=str, help="Filename containing the rules and facts"
    )
    return parser.parse_args()


def main():
    args = __parse_args()
    lines = parse_file(args.filename)
    for line in lines:
        print(line)


if __name__ == "__main__":
    main()

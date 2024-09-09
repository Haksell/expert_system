import argparse
from collections import defaultdict
from src.parse_file import parse_file, Queries, Facts, Rule
from enum import auto, Enum


class Boolean(Enum):
    FALSE = auto()
    UNDETERMINED = auto()
    TRUE = auto()


def __parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "filename", type=str, help="Filename containing the rules and facts"
    )
    return parser.parse_args()


def __bruteforce(rules, values, queries):
    return values


def main():
    args = __parse_args()
    lines = parse_file(args.filename)
    rules = []
    values = defaultdict(lambda: Boolean.UNDETERMINED)
    for line in lines:
        match type(line).__name__:
            case "Queries":
                queries = list(line)
                values = __bruteforce(rules, values, queries)
                print(values)
            case "Facts":
                values.clear()
                for fact in line:
                    values[fact] = Boolean.TRUE
            case "Rule":
                rules.append(line)


if __name__ == "__main__":
    main()

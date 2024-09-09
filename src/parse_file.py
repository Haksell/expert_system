import re
import sys


class Facts:
    def __init__(self, line):
        assert re.fullmatch(r"=[A-Z]*", line)
        self.__facts = list(line[1:])

    def __repr__(self):
        return f"""{self.__class__.__name__}('={"".join(self.__facts)}')"""


class Queries:
    def __init__(self, line):
        assert re.fullmatch(r"\?[A-Z]*", line)
        self.__queries = list(line[1:])

    def __repr__(self):
        return f"""{self.__class__.__name__}('?{"".join(self.__queries)}')"""


class Rules:
    @staticmethod
    def __parse_expression(expr):
        VALUE = r"!?[A-Z]"
        SYMBOL = r"\+|\||\^"
        if not re.fullmatch(rf"{VALUE}(({SYMBOL}){VALUE})*", expr):
            __panic("Invalid expression: {expr}")
        return re.findall(rf"{VALUE}|{SYMBOL}", expr)

    def __init__(self, line):
        parts = re.split(r"<?=>", line)
        if len(parts) != 2:
            __panic(f"Invalid line: {line}")
        lhs, rhs = parts
        self.__lhs = Rules.__parse_expression(lhs)
        self.__is_equivalent = "<=>" in line
        self.__rhs = Rules.__parse_expression(rhs)

    def __repr__(self):
        cls = self.__class__.__name__
        middle = f"{'<' if self.__is_equivalent else ''}=>"
        lhs = " ".join(self.__lhs)
        rhs = " ".join(self.__rhs)
        return f"{cls}('{lhs} {middle} {rhs}')"


def __panic(message):
    print(message)
    sys.exit(1)


def __parse_line(line):
    return (
        Facts(line)
        if line.startswith("=")
        else Queries(line)
        if line.startswith("?")
        else Rules(line)
    )


def parse_file(filename):
    try:
        lines = open(filename).read().strip().split("\n")
    except Exception as err:
        __panic(f'Failed to read "{filename}": {err}')
    lines = [re.sub(r"#.*", "", line.replace(" ", "")) for line in lines]
    lines = [line for line in lines if line]
    return [__parse_line(line) for line in lines]

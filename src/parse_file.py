import re
import sys


def __panic(message):
    print(message, file=sys.stderr)
    sys.exit(1)


def __syntax_error(message):
    print("Syntax Error: ", end="", file=sys.stderr)


class Facts:
    def __init__(self, line):
        self.__facts = list(line[1:])
        invalid_facts = [
            fact for fact in self.__facts if len(fact) != 1 or not fact.isupper()
        ]
        if invalid_facts:
            __syntax_error(f"Invalid Facts: {invalid_facts}")

    def __repr__(self):
        return f"""{self.__class__.__name__}('={"".join(self.__facts)}')"""


class Queries:
    def __init__(self, line):
        self.__queries = list(line[1:])
        invalid_queries = [
            query for query in self.__queries if len(query) != 1 or not query.isupper()
        ]
        if invalid_queries:
            __syntax_error(f"Invalid Queries: {invalid_queries}")

    def __repr__(self):
        return f"""{self.__class__.__name__}('?{"".join(self.__queries)}')"""


class Rules:
    @staticmethod
    def __parse_expression(expr):
        VALUE = r"!?[A-Z]"
        SYMBOL = r"\+|\||\^"
        if not re.fullmatch(rf"{VALUE}(({SYMBOL}){VALUE})*", expr):
            __syntax_error("Invalid expression: {expr}")
        return re.findall(rf"{VALUE}|{SYMBOL}", expr)

    def __init__(self, line):
        parts = re.split(r"<?=>", line)
        if len(parts) != 2:
            __syntax_error(f"Invalid line: {line}")
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

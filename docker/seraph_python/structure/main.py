import importlib
import inspect
import json
import sys
from pathlib import Path

BASE_DIR = Path(__file__).parent.absolute()


def main() -> None:
    args = sys.argv[1:]

    if len(args) != 3:
        raise ValueError("should have 3 args: module_name, function_name, function_args")

    module_name: str = args[0]
    function_name: str = args[1]
    function_args: list[str] = json.loads(args[2])
    parsed_args = [json.loads(arg) for arg in function_args]

    module = importlib.import_module(name=module_name)
    function = getattr(module, function_name)

    if not function or not inspect.isfunction(function):
        msg = f"{function_name} is not a function"
        raise ValueError(msg)

    result = function(*parsed_args)

    print(result, end="")


if __name__ == "__main__":
    main()

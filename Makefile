help:
	@python expert_system.py --help

clean:
	@rm -rf __pycache__ */__pycache__ */*/__pycache__
	@rm -rf .pytest_cache */.pytest_cache */*.pytest_cache

loc:
	@find . -name '*.py' | sort | xargs wc -l

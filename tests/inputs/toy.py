def create_list_w_list_comprehension(start, end):
    return [x for x in range(start, end + 1)]


if __name__ == "__main__":
    start, end = 1, 10
    result = create_list_w_list_comprehension(start, end)
    print(f"{result}")

import argparse


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-url", type=str, required=True)
    parser.add_argument("--commit-hash", type=str, required=True)
    parser.add_argument("--pr-number", type=str, required=False)
    return parser.parse_args()


def main():
    args = parse_args()

    with open("target/doc/index.html", "r") as f:
        content = f.read()

    pr_code = ""
    if args.pr_number:
        pr_code = f'<br><a href="{args.repo_url}/pull/{args.pr_number}">Pull Request {args.pr_number}</a>'

    commit_code = f'<br><a href="{args.repo_url}/commit/{args.commit_hash}">Commit <code>{args.commit_hash[:7]}</code></a>'

    content = content.replace(
        '</nav><div class="sidebar-resizer"',
        f'<div class="version">Deployed from{pr_code}{commit_code}</div></nav><div class="sidebar-resizer"',
    )

    with open("target/doc/index.html", "w") as f:
        f.write(content)


if __name__ == "__main__":
    main()

pub fn repo_name(repo: &String) -> String {
    let repo = repo.split("/").collect::<Vec<_>>();
    assert_eq!(repo.len(), 2);
    repo[1].to_string()
}

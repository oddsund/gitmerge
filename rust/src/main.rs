use std::io::{stdin};

use git2::{BranchType, Cred, FetchOptions, MergeOptions, RemoteCallbacks, Repository};

fn main() {
    // Open the Git repository in the current directory
    let repo = Repository::open(".").expect("Current directory is not a repository");

    // Get the name of the current branch
    let head = repo.head().expect("No HEAD found");
    let current_branch = head.shorthand().unwrap();

    // Ask for confirmation to merge the current branch
    let mut confirm = String::new();
    print!(
        "🤔 Are you sure you want to merge branch {}? [y/n] ",
        current_branch
    );
    stdin().read_line(&mut confirm).unwrap();
    if confirm.trim() != "y" {
        println!("🚫 Merge cancelled");
        return;
    }

    // Change to the main branch
    let main_branch = "main";
    let (object, reference) = repo
        .revparse_ext(main_branch)
        .expect("Main branch not found");

    repo.checkout_tree(&object, None)
        .expect("Failed to checkout");

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");

    // Merge the previously accepted branch
    let merge_refname = format!("refs/heads/{}", current_branch);
    let merge_ref = repo.find_reference(&merge_refname).expect("Failed to get reference");
    let merge_commit = repo.reference_to_annotated_commit(&merge_ref).expect("Failed to get commit");
    let mut merge_options = MergeOptions::new();
    merge_options.fail_on_conflict(true);
    merge_options.find_renames(true);
    let analysis = repo.merge_analysis(&[&merge_commit]).expect("Failed to analyze merge");
    if analysis.0.is_up_to_date() {
        println!("👍 Branch is already up-to-date");
        return;
    } else if !analysis.0.is_fast_forward() {
        println!("❌ Merge is not a fast-forward");
        return;
    }
    repo.merge(&[&merge_commit], Some(&mut merge_options), None).expect("Failed to merge");

    // Push the result
    let mut remote = repo.find_remote("origin").expect("Failed to find remote");
    let mut cb = RemoteCallbacks::new();
    let cred = Cred::ssh_key_from_agent("git").expect("Failed to get SSH key from agent");
    cb.credentials(|_, _, _| Ok(cred));
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(cb);
    remote.fetch(&[], Some(&mut fetch_options), None).expect("Failed to fetch");
    let refspec = format!(
        "refs/heads/{}:refs/heads/{}",
        main_branch,
        main_branch
    );
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(cb);
    remote.push(&[&refspec], Some(&mut push_options)).expect("Failed to push");

    // Delete the branch at origin
    remote.delete(&[&merge_refname]).expect("Failed to delete remote branch");

    // Delete the local branch
    let mut branch = repo.find_branch(&current_branch, BranchType::Local).expect("Failed to find branch");
    branch.delete().expect("Failed to delete local branch");

    println!("🎉 Branch {} merged successfully", current_branch);
}

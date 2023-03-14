use std::io::{stdin, stdout, Write};

use git2::{BranchType, Cred, FetchOptions, MergeOptions, RemoteCallbacks, Repository};

fn main() -> Result<(), git2::Error> {
    // Open the Git repository in the current directory
    let repo = Repository::open(".")?;

    // Get the name of the current branch
    let head = repo.head()?;
    let current_branch = head.shorthand().unwrap();

    // Ask for confirmation to merge the current branch
    let mut confirm = String::new();
    print!("🤔 Are you sure you want to merge branch {}? [y/n] ", current_branch);
    stdout().flush()?;
    stdin().read_line(&mut confirm)?;
    if confirm.trim() != "y" {
        println!("🚫 Merge cancelled");
        return Ok(());
    }

    // Change to the main branch
    let main_branch = "main";
    let main_refname = format!("refs/heads/{}", main_branch);
    let main_ref = repo.find_reference(&main_refname)?;
    let main_object = main_ref.peel_to_commit()?;
    let mut main_branch = repo.branch(main_branch, &main_object, false)?;
    main_branch.set_upstream(Some(&main_refname))?;
    repo.checkout_branch(&main_branch, None)?;

    // Merge the previously accepted branch
    let merge_refname = format!("refs/heads/{}", current_branch);
    let merge_ref = repo.find_reference(&merge_refname)?;
    let mut merge_options = MergeOptions::new();
    merge_options.fastforward_only(true);
    let analysis = repo.merge_analysis(&[&merge_ref])?;
    if analysis.0.is_up_to_date() {
        println!("👍 Branch is already up-to-date");
        return Ok(());
    } else if !analysis.0.is_fast_forward() {
        println!("❌ Merge is not a fast-forward");
        return Ok(());
    }
    repo.merge(&[&merge_ref], Some(&merge_options), None)?;

    // Push the result
    let mut remote = repo.find_remote("origin")?;
    let mut cb = RemoteCallbacks::new();
    let cred = Cred::ssh_key_from_agent("git")?;
    cb.credentials(|_, _, _| Ok(cred.clone()));
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(cb);
    remote.fetch(&[], Some(&mut fetch_options), None)?;
    let refspec = format!("refs/heads/{}:refs/heads/{}", main_branch.name().unwrap(), main_branch.name().unwrap());
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(cb);
    remote.push(&[&refspec], Some(&mut push_options))?;

    // Delete the branch at origin
    remote.delete(&[&merge_refname])?;

    // Delete the local branch
    let mut branch = repo.find_branch(&current_branch, BranchType::Local)?;
    branch.delete()?;

    println!("🎉 Branch {} merged successfully", current_branch);
    Ok(())
}


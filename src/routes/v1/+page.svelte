<div style="margin-left: 25px">
    <Button class="my-colored-button" variant="outlined" style="margin-top: 15px" on:click={() => {openproj_dialog = true}}>
        Open Project
    </Button>
    <Button class="my-colored-button" variant="outlined" style="margin-top: 15px; margin-left: 10px" on:click={() => {newproj_dialog = true}}>
        New Project
    </Button>
    <Button class="my-colored-button" variant="outlined" style="margin-top: 15px; margin-left: 10px">
        Settings
    </Button>
</div>
<LayoutGrid>
    <Cell span={6}>
        <div class="my-primary maindiv" class:elevated={true}>
            <center>
                <h1>
                    Your Changes
                </h1>
                <hr/>
                <h2>
                    {#if lclmodded == 0}
                        No changes found
                    {:else}
                        {lclmodded} changes found!
                    {/if}
                </h2>
                {#each files as file}
                    {#if ((file.status == 2 || file.status == 3 || file.status == 4 || file.status == 6 || file.status == 8))}
                        <div style="border: 1px solid {(file.status == 4 ? "red" : "#0FFF50")}">
                            <FormField>
                                <Checkbox bind:checked={file.select} />
                                <span slot="label">
                                    <p class="listtext">
                                        {file.name}
                                    </p>
                                </span>
                            </FormField>
                            <p style="margin-top: 0px; color: #0FFF50">
                                {#if file.status == 2}
                                    Modified by you
                                {:else if file.status == 3}
                                    <span style="color: red">
                                        Merge conflict: Both you and remote modified this file
                                    </span>
                                {:else if file.status == 4}
                                    <span style="color: red">
                                        Merge conflict: Both you and remote modified this file
                                    </span>
                                {:else if file.status == 6}
                                    Deleted by you
                                {:else if file.status == 8}
                                    Added by you
                                {/if}
                            </p>
                        </div><br/>
                    {/if}
                {/each}
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px">
                    Upload To Cloud
                </Button>
            </center>
        </div>
    </Cell>
    <Cell span={6}>
        <div class="my-primary maindiv" class:elevated={true}>
            <center>
                <h1>
                    Remote Changes
                </h1>
                <hr/>
                <h2>
                    {#if remotemodded == 0}
                        No changes found
                    {:else}
                        {remotemodded} changes found!
                    {/if}
                </h2>
                {#each files as file}
                    {#if ((file.status == 1 || file.status == 3 || file.status == 4 || file.status == 5 || file.status == 7))}
                        <div style="border: 1px solid {(file.status == 4 ? "red" : "#0FFF50")}">
                            <FormField>
                                <Checkbox bind:checked={file.select} />
                                <span slot="label">
                                    <p class="listtext">
                                        {file.name}
                                    </p>
                                </span>
                            </FormField>
                            <p style="margin-top: 0px; color: #0FFF50">
                                {#if file.status == 1}
                                    Modified by cloud
                                {:else if file.status == 3}
                                    <span style="color: red">
                                        Merge conflict: Both you and cloud modified this file
                                    </span>
                                {:else if file.status == 4}
                                    <span style="color: red">
                                        Merge conflict: Both you and cloud modified this file
                                    </span>
                                {:else if file.status == 5}
                                    Added by cloud
                                {:else if file.status == 7}
                                    Deleted by cloud
                                {/if}
                            </p>
                        </div><br/>
                    {/if}
                {/each}
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px">
                    Download From Cloud
                </Button>
            </center>
        </div>
    </Cell>
</LayoutGrid>

<Dialog
  bind:open={open_project_namer}
  aria-labelledby="simple-title"
  aria-describedby="simple-content"
  fullscreen
>
    <!-- Title cannot contain leading whitespace due to mdc-typography-baseline-top() -->
    <Title id="simple-title">What do you want your project to be called?</Title>
    <Content id="simple-content">
        {#if remote_sel}
            <Textfield bind:value={remote_project} label="Link" style="width: 80%">
            </Textfield>
        {:else}
            <Textfield bind:value={project} label="Link" style="width: 80%">
            </Textfield>
        {/if}
    </Content>
    <Actions>
        <Button on:click={() => create_project()}>
            <Label>Create</Label>
        </Button>
    </Actions>
</Dialog>

<Dialog
  bind:open={gd_newproj_dialog}
  aria-labelledby="simple-title"
  aria-describedby="simple-content"
  fullscreen
>
    <!-- Title cannot contain leading whitespace due to mdc-typography-baseline-top() -->
    <Title id="simple-title">Paste a link to the Google Drive folder where you want the project to be made</Title>
    <Content id="simple-content">
        <Textfield bind:value={gd_newproj_url} label="Link" style="width: 80%">
        </Textfield>
    </Content>
    <Actions>
        <Button on:click={() => newproject_gd()}>
            <Label>Create</Label>
        </Button>
    </Actions>
</Dialog>

<Dialog
  bind:open={gd_openproj_dialog}
  aria-labelledby="simple-title"
  aria-describedby="simple-content"
  fullscreen
>
    <!-- Title cannot contain leading whitespace due to mdc-typography-baseline-top() -->
    <Title id="simple-title">Paste a link to the Google Drive folder where the project is</Title>
    <Content id="simple-content">
        <Textfield bind:value={gd_newproj_url} label="Link" style="width: 80%">
        </Textfield>
    </Content>
    <Actions>
        <Button on:click={() => ldproject_gd()}>
            <Label>Create</Label>
        </Button>
    </Actions>
</Dialog>

<Dialog
  bind:open={gd_uploading}
  scrimClickAction=""
  escapeKeyAction=""
  aria-labelledby="mandatory-title"
  aria-describedby="mandatory-content"
  fullscreen
>
    <!-- Title cannot contain leading whitespace due to mdc-typography-baseline-top() -->
    <Title id="simple-title">Uploading Files To Google Drive</Title>
    <Content id="simple-content">
        <h3>
            Please wait...
        </h3>
    </Content>
</Dialog>

<Dialog
  bind:open={project_open}
  aria-labelledby="simple-title"
  aria-describedby="simple-content"
  fullscreen
>
    <!-- Title cannot contain leading whitespace due to mdc-typography-baseline-top() -->
    <Title id="simple-title">Recent Projects</Title>
    <Content id="simple-content">
        <Textfield bind:value={gd_newproj_url} label="Link" style="width: 80%">
        </Textfield>
    </Content>
    <Actions>
        <Button on:click={() => ldproject_gd()}>
            <Label>Create</Label>
        </Button>
    </Actions>
</Dialog>

<Dialog
  bind:open={newproj_dialog}
  aria-labelledby="simple-title"
  aria-describedby="simple-content"
  fullscreen
>
    <!-- Title cannot contain leading whitespace due to mdc-typography-baseline-top() -->
    <center>
        <h1>
            Where to create the project from?
        </h1>
    </center>
    <Content id="simple-content">
        <center>
            <LayoutGrid>
                <Cell span={6}>
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <div class="maindiv-hover{(newproj_sel_status == 1) ? "-sel" : ""}" class:hover-elevated={newproj_local_hover} class:elevated={newproj_local_hover}
                    on:mouseenter={() => {
                        newproj_local_hover = true;
                    }}
                    on:mouseleave={() => {
                        newproj_local_hover = false;
                    }}
                    on:click={() => {newproj_sel_status = 1}}>
                        <center>
                            <h1 style="margin-top: 5px">
                                Upload to the cloud
                            </h1>
                            <h3>
                                Project is already stored on my computer
                            </h3>
                        </center>
                    </div>
                </Cell>
                <Cell span={6}>
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <div class="maindiv-hover{(newproj_sel_status == 2) ? "-sel" : ""}" class:hover-elevated={newproj_cloud_hover} class:elevated={newproj_cloud_hover}
                    on:mouseenter={() => {
                        newproj_cloud_hover = true;
                    }}
                    on:mouseleave={() => {
                        newproj_cloud_hover = false;
                    }}
                    on:click={() => {newproj_sel_status = 2}}>
                        <center>
                            <h1 style="margin-top: 5px">
                                Download from the cloud
                            </h1>
                            <h3>
                                Project is already stored in the cloud
                            </h3>
                        </center>
                    </div>
                </Cell>
            </LayoutGrid>
            {#if newproj_sel_status == 1}
                <h1>
                    Upload this folder <br/>
                    <Button class="my-colored-button" variant="outlined" style="margin-top: 15px; margin-left: 10px" on:click={() => initFileChooser}>
                        <Icon class="material-icons">folder_open</Icon>
                        Select Folder
                    </Button>
                </h1>
                <h1>
                    To This Google Drive Link
                </h1>
                <h3 style="margin-bottom: 0px">
                    Paste the Google Drive link to folder where you want to upload this folder to
                </h3>
                <Textfield bind:value={gd_newproj_url} label="Link" style="width: 80%">
                </Textfield>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px; margin-left: 10px" on:click={() => createFromLocal}>
                    Create!
                </Button>
            {:else if newproj_sel_status == 2}
                <h1>
                    Download This Google Drive Link
                </h1>
                <h3 style="margin-bottom: 0px">
                    Paste the Google Drive link of the folder you want to download
                </h3>
                <Textfield bind:value={gd_newproj_url} label="Link" style="width: 80%">
                </Textfield>
                <h1>
                    To this folder <br/>
                    <Button class="my-colored-button" variant="outlined" style="margin-top: 15px; margin-left: 10px">
                        <Icon class="material-icons">folder_open</Icon>
                        Select Folder
                    </Button>
                </h1>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px; margin-left: 10px" on:click={() => createFromCloud}>
                    Create!
                </Button>
            {/if}
        </center>
    </Content>
</Dialog>


<Dialog
  bind:open={openproj_dialog}
  aria-labelledby="simple-title"
  aria-describedby="simple-content"
  fullscreen
>
    <!-- Title cannot contain leading whitespace due to mdc-typography-baseline-top() -->
    <center>
        <h1>
            Select a project
        </h1>
    </center>
    <Content id="simple-content">
        <Button class="my-colored-button" variant="raised" style="margin-top: 15px; margin-left: 10px; width: 100%">
            Open Project 1
        </Button>
        <Button class="my-colored-button" variant="raised" style="margin-top: 15px; margin-left: 10px; width: 100%">
            Open Project 2
        </Button>
        <Button class="my-colored-button" variant="raised" style="margin-top: 15px; margin-left: 10px; width: 100%">
            Open Project 3
        </Button>
    </Content>
    <Actions>
        <Button on:click={() => ldproject_gd()}>
            <Label>Close</Label>
        </Button>
    </Actions>
</Dialog>

<style>
    .maindiv {
        padding: 10px 10px 10px 10px;
        border: 2px solid #b300ff;
        background: rgb(0, 0, 0)
    }

    .maindiv-hover {
        padding: 10px 10px 10px 10px;
        border: 2px solid #00ddff;
        border-radius: 5px;
        background: rgb(0, 0, 0);
        transition: 0.1s border;
        cursor: pointer;
    }

    .maindiv-hover-sel {
        padding: 10px 10px 10px 10px;
        border: 2px solid #b300ff;
        border-radius: 5px;
        background: rgb(0, 0, 0);
        transition: 0.1s border;
        cursor: pointer;
    }

    .maindiv-hover:hover {
        padding: 10px 10px 10px 10px;
        border: 2px solid #b300ff;
        border-radius: 5px;
        background: rgb(0, 0, 0);
        cursor: pointer;
    }

    .listtext {
        margin-top: 0px;
        margin-bottom: 0px;
    }
</style>

<script lang="ts">
    import Card, { Content } from '@smui/card';
    import LayoutGrid, { Cell } from '@smui/layout-grid';
    import Button, { Label, Icon } from '@smui/button';
    import Textfield from '@smui/textfield';
    import CharacterCounter from '@smui/textfield/character-counter';
    import Checkbox from '@smui/checkbox';
    import FormField from '@smui/form-field';
    import HelperText from '@smui/textfield/helper-text';
    import Dialog, { Title, Actions } from '@smui/dialog';
    import Snackbar from '@smui/snackbar';
    import IconButton from '@smui/icon-button';
    import Switch from '@smui/switch';
    import SegmentedButton, { Segment } from '@smui/segmented-button';
    import Radio from '@smui/radio';
    import DataTable, { Head, Body, Row } from '@smui/data-table';

    import Tab from '@smui/tab';
    import TabBar from '@smui/tab-bar';

    import { onDestroy, onMount } from 'svelte';
    import { app, fs} from "@tauri-apps/api"
    import { open } from '@tauri-apps/api/dialog';

    import { invoke } from '@tauri-apps/api/tauri'

    //import { appConfigDir } from '@tauri-apps/api/path';
    //const appConfigDirPath = await appConfigDir();

    let choices = ['Local', 'Remote'];
    let selected = 'Local';

    let project = "None";
    let project_path = "";
    let project_dir = "";

    let remote_project = "None";
    let remote_project_path = "";
    let remote_project_dir = "";

    let activeremote = 'Google Drive';

    let remote_sel = false;

    let projectselected = false;
    let remoteprojectsel = false;
    let commitmsg = "";

    let email = "";
    let username = "";

    let logged_in = false;

    let open_project_namer = false;

    let cloud_set = false;

    let gd_auth = false;
    let gd_newproj_dialog = false;
    let gd_openproj_dialog = false;
    let gd_uploading = false;
    let gd_newproj_url = "";
    let gd_proj_dir_id = "";
    let project_open = false;
    let newproj_dialog = false;

    let newproj_sel_status = 0;

    let newproj_local_hover = false;
    let newproj_cloud_hover = false;

    let openproj_dialog = false;

    let lclmodded = 0;
    let remotemodded = 0;

    type filesel = {
        name: String,
        select: boolean,
        path: String,
        status: number
    }

    let files: filesel[] = [];

    const login = () => {
        logged_in = !logged_in;
        invoke('login', {email: email, name: username}).then((result) => {
            console.log(result);
        });
    }

    const save_changed = () => {
        let tocommit: filesel[] = [];
        files.forEach((val) => {
            if(val.select && (val.status == 2 || val.status == 3 || val.status == 4 || val.status == 6 || val.status == 8)){
                if(val)
                    tocommit.push(val);
            }
        });
        if(activeremote == 'Google Drive'){
            gd_uploading = true;
            //files: Vec<FileData>, commitmessage: String, remoteid: String, projectpath: String, projectname: String
            invoke('gd_commit', {files: tocommit, commitmessage: "", remoteid: gd_proj_dir_id, projectpath: project_dir, projectname: project}).then((result) => {
                console.log(result);
                if(result) {
                    commitmsg = "";
                }
                gd_uploading = false;
            });
        }else{
            invoke('commit', {files: tocommit, commitmessage: commitmsg, remoteproject: remote_project, remotepath: remote_project_dir, projectpath: project_dir, projectname: project}).then((result) => {
                console.log(result);
                if(result) {
                    commitmsg = "";
                }
            });
        }
    }

    const pull_changes = () => {
        let tocommit: filesel[] = [];
        files.forEach((val) => {
            if(val.select && (val.status == 1 || val.status == 3 || val.status == 4 || val.status == 5 || val.status == 7)){
                if(val)
                    tocommit.push(val);
            }
        });
        if(activeremote == 'Google Drive'){
            invoke('gd_pull', {files: tocommit, remoteid: gd_proj_dir_id, projectpath: project_dir, projectname: project}).then((result) => {
                console.log(result);
                if(result) {
                    commitmsg = "";
                }
            });
        }else{
            invoke('commit', {files: tocommit, commitmessage: "", remoteproject: project, remotepath: project_dir, projectpath: remote_project_dir, projectname: remote_project}).then((result) => {
                console.log(result);
                if(result) {
                    commitmsg = "";
                }
            });
        }
    }

    const openFileChooser = async (remote: boolean) => {
        const selected = await open({
        filters: [{
            name: 'Sync File',
            extensions: ['sync']
        }]
        });
        if (Array.isArray(selected)) {
        // user selected multiple files
        } else if (selected === null) {
        // user cancelled the selection
        } else {
            // user selected a single file
            console.log(selected);

            if(!remote){
                project_path = selected;

                let arr = selected.split("/");
                project = arr[arr.length - 1].replace(".sync", "");
                const tmp = new URL(selected, 'http://example.com'); // Assuming a base URL for relative paths
                const parentDirectory = tmp.pathname.replace(/\/[^/]+$/, '/');
                project_dir = parentDirectory;
                projectselected = true;
            }else {
                remote_project_path = selected;

                let arr = selected.split("/");
                remote_project = arr[arr.length - 1].replace(".sync", "");
                const tmp = new URL(selected, 'http://example.com'); // Assuming a base URL for relative paths
                const parentDirectory = tmp.pathname.replace(/\/[^/]+$/, '/');
                remote_project_dir = parentDirectory;
                remoteprojectsel = true;
            }
        }
    };

    const initFileChooser = async (remote: boolean) => {
        const selected = await open({
            directory: true,
        });
        if (Array.isArray(selected)) {
        // user selected multiple files
        } else if (selected === null) {
        // user cancelled the selection
        } else {
            // user selected a single file
            console.log(selected);

            remote_sel = remote;

            if(remote){
                remote_project_path = selected;
                remote_project_dir = selected;
                open_project_namer = true;

                let arr = selected.split("/");
                remote_project = arr[arr.length - 1];
            }else{
                project_path = selected;
                project_dir = selected
                open_project_namer = true;

                let arr = selected.split("/");
                project = arr[arr.length - 1];
            }
        }
    };
    let checkinterval = 0;
    onMount(() => {
        checkinterval = setInterval(() => {
            if(projectselected && remoteprojectsel){
                if(activeremote == 'Google Drive') {
                    invoke('list_files_gd', {path: project_dir, projectname: project, remoteDrive: gd_proj_dir_id}).then((result) => {
                        result.forEach(element => {
                            let res = false;
                            files.forEach(element2 => {
                                if(element2.path == element.path){
                                    res = true;
                                    element2.status = element.status;
                                }
                            });
                            if(!res){
                                files.push(element);
                            }
                            lclmodded = 0;
                            remotemodded = 0;
                            let val = element;
                            if(val.status == 2 || val.status == 3 || val.status == 4 || val.status == 6 || val.status == 8) {
                                lclmodded ++;
                            }else{
                                remotemodded ++;
                            }
                        });
                        let toremove: filesel[] = [];
                        console.log(files);
                        files.forEach(element => {
                            let found = false;
                            result.forEach(element2 => {
                                if(element2.path == element.path){
                                    found = true;
                                }
                            })
                            if(!found){
                                toremove.push(element);
                            }
                        });

                        console.log(toremove);
                        console.log(result);

                        toremove.forEach(element => {
                            files.splice(files.indexOf(element), 1);
                        });
                        files = files;
                        });
                }else{
                //path: String, projectname: String, remotepath: String, remoteproject: String
                    invoke('list_files', {path: project_dir, projectname: project, remotepath: remote_project_dir, remoteproject: remote_project}).then((result) => {
                        result.forEach(element => {
                            let res = false;
                            files.forEach(element2 => {
                                if(element2.path == element.path){
                                    res = true;
                                    element2.status = element.status;
                                }
                            });
                            if(!res){
                                files.push(element);
                            }
                        });
                        let toremove: filesel[] = [];
                        console.log(files);
                        files.forEach(element => {
                            let found = false;
                            result.forEach(element2 => {
                                if(element2.path == element.path){
                                    found = true;
                                }
                            })
                            if(!found){
                                toremove.push(element);
                            }
                        });

                        console.log(toremove);
                        console.log(result);

                        toremove.forEach(element => {
                            files.splice(files.indexOf(element), 1);
                        });
                        files = files;
                        });
                }
            }
        }, 2500)
    });

    onDestroy(() => {
        clearInterval(checkinterval);
    });

    function create_project() {
        if(remote_sel){
            invoke('initialize', {path: remote_project_path, projectname: remote_project}).then((result) => {
                    //i should prolly do smth here eventually
                    console.log(result);
                    remoteprojectsel = true;
                    open_project_namer = false;
                    remote_project_path += "/" + remote_project + ".sync";
                });
        }else{
            invoke('initialize', {path: project_path, projectname: project}).then((result) => {
                    //i should prolly do smth here eventually
                    console.log(result);
                    projectselected = true;
                    open_project_namer = false;
                    project_path += "/" + project + ".sync";
                });
        }
    }

    function auth_gd() {
        invoke('gd_auth').then((result) => {
            gd_auth = result;
        })
    }

    function newproject_gd(){
        let url = gd_newproj_url.replace(/\/$/, '');

        // Use the URL API to extract the pathname
        const pathname = new URL(url).pathname;

        // Split the pathname by slashes and return the last segment
        const segments = pathname.split('/');
        let id = segments[segments.length - 1];
        gd_proj_dir_id = id;

        invoke('gd_initialize', {path: project_dir, id: id, projectname: project}).then((result) => {
            gd_newproj_dialog = false;
            remoteprojectsel = true;
        });
    }

    function ldproject_gd(){
        let url = gd_newproj_url.replace(/\/$/, '');

        // Use the URL API to extract the pathname
        const pathname = new URL(url).pathname;

        // Split the pathname by slashes and return the last segment
        const segments = pathname.split('/');
        let id = segments[segments.length - 1];
        gd_proj_dir_id = id;

        gd_newproj_dialog = false;
        remoteprojectsel = true;
    }

    function createFromLocal(){
        create_project();
        newproject_gd();
        save_changed();
    }

    function createFromCloud(){
        create_project();
        ldproject_gd();
        pull_changes();
    }


</script>
<LayoutGrid>
    <Cell>
        <div class="my-primary maindiv" class:elevated={true}>
            <center>
                <h1>
                    Local Project
                </h1>
                <hr/>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px" on:click={() => openFileChooser(false)}>
                    Open Project
                </Button>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px" on:click={() => initFileChooser(false)}>
                    Create New Project
                </Button><br/>
                <h3>
                    Current Project: {project}
                </h3>
                <hr/>
                <h2>
                    Save Changes
                </h2>
                <div class="margins">
                    <Textfield textarea input$maxlength={250} bind:value={commitmsg} label="Change Message" disabled={!projectselected}>
                        <CharacterCounter slot="internalCounter">0 / 250</CharacterCounter>
                    </Textfield>
                </div>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px" disabled={!projectselected}>
                    Upload To Remote Project
                </Button>
            </center>
        </div>
    </Cell>
    <Cell>
        <div class="my-primary maindiv" class:elevated={true}>
            <center>
                <h1>
                    Project Files
                </h1>
                <hr/>
                {#if (files.length == 0)}
                    {#if (!projectselected)}
                        <h2>
                            No Project Selected
                        </h2>
                    {:else}
                        <h2>
                            No Files Changed
                        </h2>
                    {/if}
                {:else}
                    <h2 style="margin-bottom: 0px">
                        Modified Files
                    </h2>
                    {#each choices as option}
                        <FormField>
                            <Radio
                                bind:group={selected}
                                value={option}
                            />
                            <span slot="label">
                                {option}
                            </span>
                        </FormField>
                    {/each}
                {/if}
                {#each files as file}
                    {#if file.status != 0}
                        {#if (selected == 'Local' && (file.status == 2 || file.status == 4 || file.status == 7)) || (selected == 'Remote' && (file.status == 1 || file.status == 4 || file.status == 5))}
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
                                        Modified by remote
                                    {:else if file.status == 2}
                                        Modified by you
                                    {:else if file.status == 4}
                                        <span style="color: red">
                                            Merge conflict: Both you and remote modified this file
                                        </span>
                                    {:else if file.status == 5}
                                        Added in remote
                                    {:else if file.status == 7}
                                        Added by you
                                    {:else}
                                        {file.status}
                                    {/if}
                                </p>
                            </div><br/>
                        {/if}
                    {/if}
                {/each}
            </center>
        </div>
    </Cell>
    <Cell>
        <div class="my-primary maindiv" class:elevated={true}>
            <center>
                <h1>
                    Remote Project
                </h1>
                <hr/>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px" on:click={() => openFileChooser(true)}>
                    Open Project
                </Button>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px" on:click={() => initFileChooser(true)}>
                    Create New Project
                </Button>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px" on:click={() => {
                    remote_project = ""
                    remoteprojectsel = false;
                }}>
                    Unload Project
                </Button><br/>
                <h3>
                    Current Project: {remote_project}
                </h3>
                <hr/>
                <Button class="my-colored-button" variant="outlined" style="margin-top: 15px; margin-bottom: 15px" disabled={!remoteprojectsel}>
                    Pull Changes From Remote Project
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

<style>
    .maindiv {
        padding: 10px 10px 10px 10px;
        border: 2px solid #b300ff;
        background: rgb(0, 0, 0)
    }

    .listtext {
        margin-top: 0px;
        margin-bottom: 0px;
    }
</style>

<script lang="ts">
    import Card, { Content } from '@smui/card';
    import LayoutGrid, { Cell } from '@smui/layout-grid';
    import Button, { Label } from '@smui/button';
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

    import { onDestroy, onMount } from 'svelte';
    import { app, fs} from "@tauri-apps/api"
    import { open } from '@tauri-apps/api/dialog';

    import { invoke } from '@tauri-apps/api/tauri'

    let choices = ['Local', 'Remote'];
    let selected = 'Local';

    let project = "None";
    let project_path = "";
    let project_dir = "";

    let remote_project = "None";
    let remote_project_path = "";
    let remote_project_dir = "";

    let remote_sel = false;

    let projectselected = false;
    let remoteprojectsel = false;
    let commitmsg = "";

    let email = "";
    let username = "";

    let logged_in = false;

    let open_project_namer = false;

    let cloud_set = false;

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
        let tocommit: String[] = [];
        files.forEach((val) => {
            if(val.select){
                tocommit.push(val.path);
            }
        });

        invoke('commit', {files: tocommit, commitMessage: commitmsg}).then((result) => {
            console.log(result);
            if(result) {
                commitmsg = "";
            }
        });
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
                //path: String, projectname: String, remotepath: String, remoteproject: String
                invoke('list_files', {path: project_dir, projectname: project, remotepath: remote_project_dir, remoteproject: remote_project}).then((result) => {
                        files = result;
                    });
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


</script>
if (window.viz_fsm_inline === true) {
    $("#viz_fullscreen_link").attr("href", window.viz_html_full);
    var cy = init_cy_fsm();
    var f = viz_fsm_body(cy);
    f.run_layout();
}

function init_cy_fsm() {
    var cy = window.cy = cytoscape({
        container: document.getElementById('cy'),

        boxSelectionEnabled: false,
        autounselectify: true,

        style: [
            {
            selector: 'node',
            css: {
                'label': 'data(label)',
                'width': 'label',
                'height': 'label',
                'padding': 20,
                'text-valign': 'center',
                'text-halign': 'center',
                'border-color': '#50514F',
                'border-width': '1',
                'background-color': '#FFE066'
            }
            },
            {
            selector: 'node.start',
            css: {
                'shape': 'hexagon',
                'border-style': 'dashed'
            }
            },
            {
            selector: 'node.initial',
            css: {
                'border-width': 10,
                'border-style': 'double'
            }
            },
            {
            selector: 'node.interrupt',
            css: {
                'background-color': '#F25F5C'
            }
            },
            {
            selector: '$node > node',
            css: {
                'padding-top': '10px',
                'padding-left': '10px',
                'padding-bottom': '10px',
                'padding-right': '10px',
                'text-valign': 'top',
                'text-halign': 'center',
                'background-color': '#247BA0',
                'background-opacity': '0.2',
                'border-color': '#888',
                'border-width': '1'
                //'border': '1px solid #bbb'
                /* 'background-color': '#bbb' */
            }
            },
            {
            selector: 'edge',
            css: {
                label: "data(label)",
                'text-wrap': 'wrap',
                
                'curve-style': 'bezier',
                'control-point-step-size': 200,

                //'text-margin-y': -50,
                'text-outline-color': 'white',
                'text-outline-width': 2,

                'width': 2,
                'color': '#333',
                'line-color': '#50514F',
                'source-arrow-shape': 'circle',
                'target-arrow-color': '#F25F5C',
                'target-arrow-shape': 'triangle',
                //'source-endpoint': 'outside-to-line',
                //'target-endpoint': 'outside-to-line'
            }
            },
            {
                selector: 'edge.start',
                css: {
                    'line-style': 'dotted'
                }
            },
            {
                selector: 'edge.fsm',
                css: {
                    'target-endpoint': 'outside-to-node'
                }
            },
            {
                selector: 'edge.internal_transition',
                css: {
                    'line-style': 'dotted'
                }
            },
            {
            selector: ':selected',
            css: {
                'background-color': 'black',
                'line-color': 'black',
                'target-arrow-color': 'black',
                'source-arrow-color': 'black'
            }
            }
        ]
    });
    return cy;
}


function newFsm(cy, name, parent) {
    var fsm_id = "fsm_" + name;
    cy.add({group: "nodes", data: { id: fsm_id, label: name, parent: parent } });
    
    var regions = [];

    return {
        fsm_id: fsm_id,
        node_id: fsm_id,
        regions: regions,

        run_layout: function() {
            var layout = cy.makeLayout({
                name: 'breadthfirst',
                directed: true,
                //padding: 100,
                nodeDimensionsIncludeLabels: true
            });

            layout.run();
        },

        add_transition_to_start(state_start, data) {
            regions.forEach(function(val) {
                val.add_transition(state_start, val.start, data);
            });
        },

        add_region: function(region) {
            var region_id = fsm_id + "_" + region;
            var start_id = region_id + "__start";            
            cy.add({group: "nodes", data: { id: region_id, label: region, parent: fsm_id }});
            cy.add({group: "nodes", data: { id: start_id, label: "Start", parent: region_id }, classes: "start" });
            var reg =  {
                region_id: region_id,
                node_id: region_id,
                
                start: {
                    node_id: start_id
                },

                add_initial_state: function(name) {
                    var initial_state_id = fsm_id + "_" + name;
                    cy.add({group: "nodes", data: { id: initial_state_id, label: name, parent: region_id }, classes: "initial" });
                    cy.add({group: "edges", data: { source: start_id, target: initial_state_id }, classes: "start" });
                    return {
                        state_id: initial_state_id,
                        node_id: initial_state_id
                    };
                },
                add_state: function(name, info) {
                    var state_id = region_id + "_" + name;
                    var node = {group: "nodes", data: { id: state_id, label: name, parent: region_id }, classes: ""};
                    if (info.is_interrupt_state) {
                        node.classes += "interrupt";
                    }

                    cy.add(node);
                    return {
                        state_id: state_id,
                        node_id: state_id
                    };
                },
                add_transition(state_start, state_target, info) {

                    //var label = "Event: <b>" + info.event + "</b>, Action: <b>" + info.action + "</b>";

                    var label = "";
                    if (info.guard != "") {
                        label += "[" + info.guard + "]\n";
                    }
                    if (info.is_anonymous == true) {
                        label += "Anonymous transition\n";
                    } else {
                        label += info.event + "\n";
                    }

                    if (info.action != "") {
                        label += "/" + info.action + "\n";
                    }
                    if (info.shallow_history == true) {
                        label += "(H)\n";
                    }
                    if (info.resume_event == true) {
                        label += "(Resume)\n";
                    }


                    var data = { source: state_start.node_id, target: state_target.node_id, label: label.trim() };
                    var edge = {group: "edges", data: data, classes: "" };

                    if (state_target.fsm_id !== undefined) {
                        edge.classes += 'fsm';
                    }
                    if (info.transition_type == "Internal") {
                        edge.classes += "internal_transition";
                    }


                    cy.add(edge);
                }
            };

            regions.push(reg);

            return reg;
        }
    }
}

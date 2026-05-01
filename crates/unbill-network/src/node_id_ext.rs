use unbill_model::NodeId;

/// Extension trait on `NodeId` to convert to the network-layer type.
pub trait NodeIdExt {
    fn to_endpoint_id(&self) -> Result<iroh::EndpointId, iroh::KeyParsingError>;
}

/// Extension trait on `iroh::EndpointId` to convert to the model-layer type.
pub trait EndpointIdExt {
    fn to_node_id(&self) -> NodeId;
}

impl NodeIdExt for NodeId {
    fn to_endpoint_id(&self) -> Result<iroh::EndpointId, iroh::KeyParsingError> {
        self.as_str().parse()
    }
}

impl EndpointIdExt for iroh::EndpointId {
    fn to_node_id(&self) -> NodeId {
        NodeId::new(self.to_string())
    }
}

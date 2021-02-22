import React from "react";
import "./../../App.less";
import { Layout } from "antd";
import { Link } from "react-router-dom";
import { LABELS } from "../../constants";
import { AppBar } from "../AppBar";

const { Header, Content } = Layout;
const tenderizeLogo = require("../../img/tenderizeLogo.svg");

export const AppLayout = React.memo((props: any) => {
  
  return (
    <div className="App wormhole-bg">
      
      <Layout title={LABELS.APP_TITLE}>
      <div className="background"></div>
      <div className="bg-container">
        <div className="bg-inner"></div>
      </div>
        <Header className="App-Bar" style={{marginTop: "20px", marginBottom: "20px"}}>
          <Link to="/">
            <div className="app-title">
            <img src={tenderizeLogo} alt="tenderize logo" />
            </div>
          </Link>
          <AppBar />
        </Header>
        <Content style={{ padding: "0 50px" }}>{props.children}</Content>
      </Layout>
    </div>
  );
});

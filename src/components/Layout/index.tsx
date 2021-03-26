import React from "react";
import "./../../App.less";
import { Layout, Menu } from "antd";
import { Link } from "react-router-dom";
import { AppBar } from "../AppBar";

const { Header, Content } = Layout;
const tenderizeLogo = require("../../img/tenderizeLogo.svg");

export const AppLayout = React.memo((props: any) => {
  return (
    <div className="App wormhole-bg">
      <div
        style={{ background: "#FFE8B9", height: "50px", paddingTop: "10px" }}
      >
        <h1 className="announcement">
          Marinade.finance won 3rd place in recent Solana x Serum DeFi
          Hackathon.{" "}
          <a
            href="https://discord.gg/CNjTngWJ"
            target="_blank"
            rel="noopener noreferrer"
          >
            <span className="text-success text-decoration-underline">
              We're hiring
            </span>
          </a>{" "}
          too!
        </h1>
      </div>
      <Layout>
        <div className="background"></div>
        <div className="bg-container">
          <div className="bg-inner"></div>
        </div>

        <Header
          className="App-Bar"
          style={{ marginTop: "20px", marginBottom: "20px" }}
        >
          <Link to="/">
            <div className="app-title">
              {/* <img src={tenderizeLogo} alt="tenderize logo" /> */}
              <h1>Marinade</h1>
            </div>
          </Link>
          <Menu mode="horizontal">
            <Menu.Item key="discord" color="#000000">
              <a
                href="https://discord.gg/CNjTngWJ"
                target="_blank"
                rel="noopener noreferrer"
              >
                Discord
              </a>
            </Menu.Item>

            <Menu.Item key="twitter">
              <a
                href="https://medium.com/tenderize-me"
                target="_blank"
                rel="noopener noreferrer"
              >
                Medium
              </a>
            </Menu.Item>
            <Menu.Item key="medium">
              <a
                href="https://twitter.com/tenderize_me"
                target="_blank"
                rel="noopener noreferrer"
              >
                Twitter
              </a>
            </Menu.Item>
          </Menu>
          <AppBar />
        </Header>
        <Content style={{ padding: "0 50px" }}>{props.children}</Content>
      </Layout>
    </div>
  );
});
